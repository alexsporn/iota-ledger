use ledger_transport::APDUCommand;

use crate::{
    Transport,
    api::{
        constants, errors, helpers,
        packable::{Error as PackableError, Read, Unpackable},
    },
};
// dashboard:
// HID => b001000000
// HID <= 0105|424f4c4f53|05|322e302e30|9000
// B O L O S      2 . 0 . 0
//
// "IOTA"
// HID => b001000000
// HID <= 0104|494f5441|05|302e372e30|0102|9000
// I O T A      0 . 7 . 0

#[derive(Debug)]
pub struct Response {
    pub app: String,
    pub version: String,
}

impl Unpackable for Response {
    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        // format always 0x01 but don't insist on it
        let _format_id = u8::unpack(buf)?;

        let app = String::unpack(buf)?;
        let version = String::unpack(buf)?;

        // consume all extra bytes (nano x <-> nano s compatibility!)
        while u8::unpack(buf).is_ok() {
            // NOP
        }

        Ok(Self { app, version })
    }
}

pub fn exec(transport: &Transport) -> Result<Response, errors::LedgerError> {
    let cmd = APDUCommand {
        cla: constants::APDU_BOLOS_CLA_B0,
        ins: constants::APDUInstructionsBolos::GetAppVersionB0 as u8,
        p1: 0,
        p2: 0,
        data: Vec::new(),
    };
    helpers::exec::<Response>(transport, cmd)
}
