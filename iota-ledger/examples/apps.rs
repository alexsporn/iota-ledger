use clap::{Arg, Command};
use std::{thread, time};

use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("ledger iota tester")
        .version("1.0")
        .author("Thomas Pototschnig <microengineer18@gmail.com>")
        .arg(
            Arg::new("is-simulator")
                .short('s')
                .long("simulator")
                .value_name("is_simulator")
                .help("select the simulator as transport")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .get_matches();

    let is_simulator = matches.get_flag("is-simulator");

    let transport_type = if is_simulator {
        iota_ledger::TransportTypes::TCP
    } else {
        iota_ledger::TransportTypes::NativeHID
    };

    let ledger: iota_ledger::LedgerHardwareWallet =
        iota_ledger::get_ledger_by_type(transport_type)?;

    if ledger.is_app_open()? {
        println!("App is already open");
    } else {
        ledger.bolos_open_app()?;
        thread::sleep(time::Duration::from_secs(5));
    }
    let version = ledger.get_version()?;
    println!("current app version: {version}");
    Ok(())
}
