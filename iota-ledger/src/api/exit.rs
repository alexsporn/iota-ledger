use crate::Transport;
use ledger_transport::APDUCommand;

use crate::api::{constants, errors, helpers};

pub fn exec(transport: &Transport) -> Result<(), errors::LedgerError> {
    let cmd = APDUCommand {
        cla: constants::APDU_BOLOS_CLA_B0,
        ins: constants::APDUInstructionsBolos::AppExitB0 as u8,
        p1: 0,
        p2: 0,
        data: Vec::new(),
    };
    helpers::exec::<()>(transport, cmd)
}
