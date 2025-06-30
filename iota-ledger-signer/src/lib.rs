use fastcrypto::ed25519::Ed25519PublicKey;
use iota_ledger::LedgerHardwareWallet;
use iota_sdk::IotaClient;
use iota_sdk::types::{base_types::IotaAddress, transaction::TransactionData};
use iota_types::crypto::Ed25519IotaSignature;
use iota_types::crypto::ToFromBytes;
use serde::Serialize;
use shared_crypto::intent::{Intent, IntentMessage};

mod utils;

pub struct IotaLedgerSigner {
    pub client: Option<IotaClient>,
    pub path: bip32::DerivationPath,
    pub ledger: LedgerHardwareWallet,
}

pub struct SignedTransaction<T> {
    pub intent_msg: IntentMessage<T>,
    pub signature: Ed25519IotaSignature,
    pub address: IotaAddress,
}

impl IotaLedgerSigner {
    pub fn new(
        ledger: LedgerHardwareWallet,
        path: bip32::DerivationPath,
        client: Option<IotaClient>,
    ) -> Self {
        IotaLedgerSigner {
            ledger,
            path,
            client,
        }
    }

    fn get_signature_scheme(&self) -> iota_types::crypto::SignatureScheme {
        iota_types::crypto::SignatureScheme::ED25519
    }

    pub fn get_address(&self) -> Result<IotaAddress, anyhow::Error> {
        let public_key = self.ledger.get_public_key(&self.path)?;
        Ok(IotaAddress::from_bytes(public_key.address)?)
    }

    pub fn get_public_key(&self) -> Result<Ed25519PublicKey, anyhow::Error> {
        let public_key = self.ledger.get_public_key(&self.path)?;
        Ok(public_key.public_key)
    }

    fn sign_intent<T: Serialize>(
        &self,
        intent_msg: IntentMessage<T>,
        bcs_objects: Vec<Vec<u8>>,
    ) -> Result<SignedTransaction<T>, anyhow::Error> {
        let public_key = self.ledger.get_public_key(&self.path)?;
        let intent_bytes = bcs::to_bytes(&intent_msg)?;

        let signature = self
            .ledger
            .sign_transaction(&self.path, intent_bytes, bcs_objects)?;

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature_bytes.extend_from_slice(&[self.get_signature_scheme().flag()]);
        signature_bytes.extend_from_slice(&signature.bytes);
        signature_bytes.extend_from_slice(public_key.public_key.as_ref());

        Ok(SignedTransaction {
            intent_msg,
            signature: Ed25519IotaSignature::from_bytes(&signature_bytes)?,
            address: IotaAddress::from_bytes(public_key.address)?,
        })
    }

    pub async fn sign_transaction(
        &self,
        transaction: TransactionData,
    ) -> Result<SignedTransaction<TransactionData>, anyhow::Error> {
        let objects = if let Some(client) = &self.client {
            utils::load_objects_with_client(client, &transaction).await?
        } else {
            vec![]
        };

        let intent_msg = IntentMessage::new(Intent::iota_transaction(), transaction);
        self.sign_intent(intent_msg, objects)
    }

    pub fn sign_message(
        &self,
        message: Vec<u8>,
    ) -> Result<SignedTransaction<Vec<u8>>, anyhow::Error> {
        let intent_msg: IntentMessage<Vec<u8>> =
            IntentMessage::new(Intent::personal_message(), message);
        self.sign_intent(intent_msg, vec![])
    }
}
