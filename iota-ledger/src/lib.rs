//! Library

use std::vec;

use log::debug;
pub use transport::{LedgerTransport, Transport, TransportTypes, create_transport};

pub use crate::api::errors::LedgerError;
use crate::api::{
    get_public_key, get_public_key::PublicKeyResult, get_version::Version, sign_transaction,
};
pub mod api;

pub mod transport;
use iota_types::{
    base_types::IotaAddress,
    crypto::{Ed25519IotaSignature, SignatureScheme, ToFromBytes},
    object::Object,
};
use serde::Serialize;
use shared_crypto::intent::IntentMessage;

/// Get Ledger by transport_type
pub fn get_ledger_by_type(
    transport_type: TransportTypes,
) -> Result<LedgerHardwareWallet, LedgerError> {
    let transport = create_transport(transport_type, None)?;
    Ok(crate::LedgerHardwareWallet::new(transport))
}

pub struct LedgerHardwareWallet {
    transport: Transport,
}

pub struct SignedTransaction<T> {
    pub intent_msg: IntentMessage<T>,
    pub signature: Ed25519IotaSignature,
    pub address: IotaAddress,
}

impl LedgerHardwareWallet {
    fn new(transport: Transport) -> Self {
        LedgerHardwareWallet { transport }
    }

    /// Get currently opened app
    /// If "BOLOS" is returned, the dashboard is open
    pub fn is_app_open(&self) -> Result<bool, LedgerError> {
        let app = crate::api::bolos_app_get_name::exec(&self.transport)?;
        Ok(app.app == "IOTA")
    }

    /// Open app on the nano s/x
    /// Only works if dashboard is open
    pub fn bolos_open_app(&self) -> Result<(), LedgerError> {
        crate::api::bolos_app_open::exec(&self.transport, "IOTA".to_string())
    }

    /// Close current opened app on the nano s/x
    /// Only works if an app is open
    pub fn bolos_exit_app(&self) -> Result<(), LedgerError> {
        crate::api::bolos_app_exit::exec(&self.transport)
    }

    fn transport(&self) -> &Transport {
        &self.transport
    }

    pub fn get_version(&self) -> Result<Version, LedgerError> {
        let version = crate::api::get_version::exec(&self.transport)?;

        debug!(
            "Connected Ledger app version: {}.{}.{}",
            version.major, version.minor, version.patch
        );

        Ok(version)
    }

    pub fn verify_address(
        &self,
        bip32: &bip32::DerivationPath,
    ) -> Result<PublicKeyResult, LedgerError> {
        get_public_key::exec(&self.transport, bip32, true)
    }

    pub fn get_public_key(
        &self,
        bip32: &bip32::DerivationPath,
    ) -> Result<PublicKeyResult, LedgerError> {
        get_public_key::exec(&self.transport, bip32, false)
    }

    pub fn get_signature_scheme(&self) -> SignatureScheme {
        SignatureScheme::ED25519
    }

    pub fn sign_intent<T: Serialize>(
        &self,
        bip32: &bip32::DerivationPath,
        intent_msg: IntentMessage<T>,
        objects: Vec<Object>,
    ) -> Result<SignedTransaction<T>, LedgerError> {
        let version = self.get_version()?;
        let public_key = self.get_public_key(bip32)?;
        let intent_bytes = bcs::to_bytes(&intent_msg).map_err(|_| LedgerError::Serialization)?;

        let signature = (if version.major > 0 {
            let bcs_objects: Vec<Vec<u8>> = objects
                .iter()
                .map(|o| bcs::to_bytes(&o).map_err(|_| LedgerError::Serialization))
                .collect::<Result<_, _>>()?;
            // If the major version is greater than 0, we assume it supports clear signing
            sign_transaction::exec(self.transport(), bip32, intent_bytes, bcs_objects)
        } else {
            sign_transaction::exec(self.transport(), bip32, intent_bytes, vec![])
        })?;

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature_bytes.extend_from_slice(&[self.get_signature_scheme().flag()]);
        signature_bytes.extend_from_slice(&signature.bytes);
        signature_bytes.extend_from_slice(public_key.public_key.as_ref());

        Ok(SignedTransaction {
            intent_msg,
            signature: Ed25519IotaSignature::from_bytes(&signature_bytes)
                .map_err(|_| LedgerError::Serialization)?,
            address: IotaAddress::from_bytes(public_key.address)
                .map_err(|_| LedgerError::Serialization)?,
        })
    }
}
