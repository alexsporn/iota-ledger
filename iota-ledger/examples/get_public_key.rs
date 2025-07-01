use std::{error::Error, str::FromStr};

use clap::{Arg, Command};

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
            Arg::new("verify")
                .long("verify")
                .help("verify address (default false)")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("is-simulator")
                .short('s')
                .long("simulator")
                .help("select the simulator as transport")
                .action(clap::ArgAction::SetTrue)
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

    let verify = matches.get_flag("verify");

    let transport_type = if is_simulator {
        iota_ledger::TransportTypes::TCP
    } else {
        iota_ledger::TransportTypes::NativeHID
    };

    let ledger = iota_ledger::get_ledger_by_type(transport_type)?;

    // generate address without prompt
    let addresses = if verify {
        ledger.verify_address(&derivation_path)?
    } else {
        ledger.get_public_key(&derivation_path)?
    };

    println!("Public Key: {}", hex::encode(&addresses.public_key));
    println!("Address: {}", hex::encode(addresses.address));

    Ok(())
}
