use clap::{Arg, Command};
use std::str::FromStr;

use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
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
            Arg::new("transaction")
                .long("tx")
                .help("transaction bytes in base64 format")
                .required(true),
        )
        .arg(
            Arg::new("is-simulator")
                .short('s')
                .long("simulator")
                .help("select the simulator as transport")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("objects")
                .long("objects")
                .help("A list of input objects in base64 format")
                .value_name("OBJECTS")
                .num_args(1..)
                .action(clap::ArgAction::Append)
                .required(false),
        )
        .get_matches();

    let is_simulator = matches.get_flag("is-simulator");

    let derivation_path = bip32::DerivationPath::from_str(
        matches
            .get_one::<String>("bip32-path")
            .map(|s| s.as_str())
            .unwrap_or("m/44'/4218'/0'/0'/0'"),
    )?;

    let objects: Vec<Vec<u8>> = matches
        .get_many::<String>("objects")
        .map(|objs| {
            objs.map(|s| {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
                    .expect("Invalid base64 in objects")
            })
            .collect()
        })
        .unwrap_or_default();

    let transport_type = if is_simulator {
        iota_ledger::TransportTypes::TCP
    } else {
        iota_ledger::TransportTypes::NativeHID
    };

    let transaction = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        matches
            .get_one::<String>("transaction")
            .expect("Transaction bytes are required"),
    )?;

    let ledger = iota_ledger::get_ledger_by_type(transport_type)?;

    let signature = ledger.sign_transaction(&derivation_path, transaction, objects)?;
    println!(
        "Signature: {}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &signature.bytes)
    );

    Ok(())
}
