use fastcrypto::ed25519::Ed25519PublicKey;
use iota_ledger::{LedgerHardwareWallet, SignedTransaction};
use iota_sdk::{
    IotaClient,
    types::{base_types::IotaAddress, crypto::SignatureScheme, transaction::TransactionData},
};
use shared_crypto::intent::{Intent, IntentMessage};
mod utils;

pub struct IotaLedgerSigner {
    pub client: Option<IotaClient>,
    pub path: bip32::DerivationPath,
    pub ledger: LedgerHardwareWallet,
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

    pub fn get_signature_scheme(&self) -> SignatureScheme {
        self.ledger.get_signature_scheme()
    }

    pub fn get_address(&self) -> Result<IotaAddress, anyhow::Error> {
        let public_key = self.ledger.get_public_key(&self.path)?;
        Ok(public_key.address)
    }

    pub fn get_public_key(&self) -> Result<Ed25519PublicKey, anyhow::Error> {
        let public_key = self.ledger.get_public_key(&self.path)?;
        Ok(public_key.public_key)
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
        self.ledger
            .sign_intent(&self.path, intent_msg, objects)
            .map_err(anyhow::Error::from)
    }

    pub fn sign_message(
        &self,
        message: Vec<u8>,
    ) -> Result<SignedTransaction<Vec<u8>>, anyhow::Error> {
        let intent_msg: IntentMessage<Vec<u8>> =
            IntentMessage::new(Intent::personal_message(), message);
        self.ledger
            .sign_intent(&self.path, intent_msg, vec![])
            .map_err(anyhow::Error::from)
    }
}
