use crate::api::packable::{Error as PackableError, Packable, Write};

use crate::Transport;
use ledger_transport::APDUCommand;

use crate::api::{constants, errors, helpers};

#[derive(Debug)]
pub struct Request {
    pub app: String,
}

impl Packable for Request {
    fn packed_len(&self) -> usize {
        self.app.packed_len()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        self.app.pack(buf)?;
        Ok(())
    }
}

pub fn exec(transport: &Transport, app: String) -> Result<(), errors::LedgerError> {
    let req = Request { app };

    let mut buf = Vec::new();
    let _ = req.pack(&mut buf);

    // string serializer stores a length byte that is unwanted here because
    // the p3 parameter will be the length of the string and the data itself
    // must not contain the length
    buf.remove(0);

    let cmd = APDUCommand {
        cla: constants::APDU_BOLOS_CLA_E0,
        ins: constants::APDUInstructionsBolos::OpenAppE0 as u8,
        p1: 0,
        p2: 0,
        data: buf,
    };
    helpers::exec::<()>(transport, cmd)
}
