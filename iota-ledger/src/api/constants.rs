pub const HARDENED: u32 = 0x80000000;

pub enum APDUInstructions {
    GetVersion = 0x00,
    VerifyAddress = 0x01,
    GetPublicKey = 0x02,
    SignTransaction = 0x03,

    GetAppConfig = 0x10,
    SetAccount = 0x11,

    Exit = 0xff,
}

pub(crate) const APDU_CLA: u8 = 0x00;
pub(crate) const APDU_P1: u8 = 0x00;
pub(crate) const APDU_P2: u8 = 0x00;

pub(crate) const APDU_BOLOS_CLA_B0: u8 = 0xb0;
pub(crate) const APDU_BOLOS_CLA_E0: u8 = 0xe0;

pub(crate) enum APDUInstructionsBolos {
    GetAppVersionB0 = 0x01,
    AppExitB0 = 0xa7,

    OpenAppE0 = 0xd8,
}
