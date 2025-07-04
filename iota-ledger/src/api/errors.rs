pub use ledger_transport::APDUErrorCode;
use thiserror::Error;

#[derive(Debug)]
#[repr(u8)]
pub enum SyscallError {
    InvalidParameter = 2,
    Overflow,
    Security,
    InvalidCrc,
    InvalidChecksum,
    InvalidCounter,
    NotSupported,
    InvalidState,
    Timeout,
    Unspecified,
}

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Unknown error")]
    Unknown,

    #[error("Device panic")]
    DevicePanic,

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Device locked")]
    DeviceLocked,

    #[error("Syscall error: {0:?}")]
    Syscall(SyscallError),

    #[error("APDU error: {0:?}")]
    APDUError(APDUErrorCode),

    #[error("Blocks protocol failed")]
    BlocksProtocolFailed,

    #[error("Transport error")]
    TransportError,

    #[error("Serialization error")]
    Serialization,

    #[error("Timeout")]
    Timeout,
}

impl LedgerError {
    /// Convert a raw error code to an LedgerError.
    ///
    /// This method first tries to match standard APDU error codes using
    /// APDUErrorCode::try_from(). If that fails, it falls back to matching
    /// legacy/custom error codes that are specific to this application but
    /// not part of the standard APDU protocol.
    ///
    /// Standard APDU errors (0x6xxx range) will be wrapped in
    /// APDUError(APDUErrorCode). Custom application errors (like 0x5515 for
    /// DeviceLocked, 0xe000 for Panic) are handled separately to maintain
    /// backward compatibility.
    pub fn get_error(rc: u16) -> Option<LedgerError> {
        // First try to match APDU error codes
        if let Ok(apdu_error) = APDUErrorCode::try_from(rc) {
            match apdu_error {
                APDUErrorCode::NoError => return None, // No error, return None
                _ => return Some(LedgerError::APDUError(apdu_error)),
            }
        }

        // Fall back to legacy error code matching for non-standard codes
        let e = match rc {
            0xe000 => LedgerError::DevicePanic,
            0x5515 => LedgerError::DeviceLocked,
            rc if (0x6800..=0x680b).contains(&rc) => {
                let value = (rc - 0x6800) as u8;
                let syscall_error = match value {
                    2 => SyscallError::InvalidParameter,
                    3 => SyscallError::Overflow,
                    4 => SyscallError::Security,
                    5 => SyscallError::InvalidCrc,
                    6 => SyscallError::InvalidChecksum,
                    7 => SyscallError::InvalidCounter,
                    8 => SyscallError::NotSupported,
                    9 => SyscallError::InvalidState,
                    10 => SyscallError::Timeout,
                    _ => SyscallError::Unspecified,
                };
                LedgerError::Syscall(syscall_error)
            }
            _ => LedgerError::Unknown,
        };
        Some(e)
    }
}
