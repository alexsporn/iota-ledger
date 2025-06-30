use clap::{Arg, Command};
use iota_sdk::{IotaClientBuilder, types::transaction::TransactionData};
use iota_types::crypto::{EncodeDecodeBase64, IotaSignature};
use std::str::FromStr;

fn transaction_from_base64(b64: &str) -> Result<TransactionData, anyhow::Error> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)?;
    bcs::from_bytes(&bytes).map_err(anyhow::Error::from)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let matches = Command::new("iota-ledger-cli")
        .version("1.0")
        .arg(
            Arg::new("bip32-path")
                .short('p')
                .long("path")
                .help("bip32 path to use (default \"m/44'/4218'/0'/0'/0'\")")
                .value_name("PATH")
                .required(false),
        )
        .arg(
            Arg::new("network")
                .short('n')
                .long("network")
                .help("select the network to connect to for fetching inputs (local, devnet, testnet, mainnet or custom URL)")
                .required(false),
        )
        .arg(
            Arg::new("transaction")
                .long("tx")
                .help("transaction bytes in base64 format")
                .required(true),
        )
        .get_matches();

    let derivation_path = bip32::DerivationPath::from_str(
        matches
            .get_one::<String>("bip32-path")
            .map(|s| s.as_str())
            .unwrap_or("m/44'/4218'/0'/0'/0'"),
    )?;

    let transaction = transaction_from_base64(
        matches
            .get_one::<String>("transaction")
            .expect("Transaction bytes are required"),
    )?;

    let network = matches.get_one::<String>("network").map(|s| s.as_str());
    let client = match network {
        Some("local") => Some(IotaClientBuilder::default().build_localnet().await?),
        Some("devnet") => Some(IotaClientBuilder::default().build_devnet().await?),
        Some("testnet") => Some(IotaClientBuilder::default().build_testnet().await?),
        Some("mainnet") => Some(IotaClientBuilder::default().build_mainnet().await?),
        Some(url) => Some(IotaClientBuilder::default().build(url).await?),
        None => None,
    };
    if let Some(c) = &client {
        println!(
            "Connected to IOTA network: {} using version {}",
            network.unwrap(),
            c.api_version()
        );
    } else {
        println!("No IOTA network specified, only blind-signing supported.");
    }

    let ledger = iota_ledger::get_ledger_by_type(iota_ledger::TransportTypes::NativeHID)?;

    let signer = iota_ledger_signer::IotaLedgerSigner::new(ledger, derivation_path, client);

    let signed_tx = signer.sign_transaction(transaction).await?;
    println!("Signature: {}", signed_tx.signature.encode_base64());

    // Verify the signature
    signed_tx.signature.verify_secure(
        &signed_tx.intent_msg,
        signer.get_address()?,
        signed_tx.signature.scheme(),
    )?;

    Ok(())
}
