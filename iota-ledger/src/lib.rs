//! Library

use std::vec;

use log::debug;

pub use ledger_transport::{APDUAnswer, APDUCommand, APDUErrorCode};

use crate::api::errors::LedgerError;
use crate::api::get_public_key;
use crate::api::get_version::Version;
use crate::api::sign_transaction::SignatureBytes;

use crate::api::get_public_key::PublicKeyResult;
pub mod api;

pub mod transport;
pub use transport::{LedgerTransport, Transport, TransportTypes, create_transport};

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
    ) -> Result<PublicKeyResult, api::errors::LedgerError> {
        get_public_key::exec(&self.transport, bip32, true)
    }

    pub fn get_public_key(
        &self,
        bip32: &bip32::DerivationPath,
    ) -> Result<PublicKeyResult, api::errors::LedgerError> {
        get_public_key::exec(&self.transport, bip32, false)
    }

    pub fn sign_transaction(
        &self,
        bip32: &bip32::DerivationPath,
        transaction: Vec<u8>,
        objects: Vec<Vec<u8>>,
    ) -> Result<SignatureBytes, api::errors::LedgerError> {
        let version = self.get_version()?;
        if version.major > 0 {
            // If the major version is greater than 0, we assume it supports clear signing
            api::sign_transaction::exec(self.transport(), bip32, transaction, objects)
        } else {
            api::sign_transaction::exec(self.transport(), bip32, transaction, vec![])
        }
    }
}
