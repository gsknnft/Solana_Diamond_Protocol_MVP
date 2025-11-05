/*!
 * Diamond Error Types - Native Rust Implementation
 * 
 * Error codes matching Anchor version for compatibility.
 */

use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Diamond-specific errors
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[repr(u32)]
pub enum DiamondError {
    /// Module not found for the given selector
    #[error("Module not found")]
    ModuleNotFound = 6000,

    /// Unauthorized access attempt
    #[error("Unauthorized access")]
    UnauthorizedAccess = 6001,

    /// Invalid selector (wrong size or format)
    #[error("Invalid selector")]
    InvalidSelector = 6002,

    /// Module capacity exceeded (max 20)
    #[error("Module capacity exceeded")]
    ModuleCapacityExceeded = 6003,

    /// Selector capacity exceeded (max 50)
    #[error("Selector capacity exceeded")]
    SelectorCapacityExceeded = 6004,

    /// Selector collision detected
    #[error("Selector collision")]
    SelectorCollision = 6005,

    /// Diamond is paused
    #[error("Diamond is paused")]
    DiamondPaused = 6006,

    /// Selector is marked as immutable
    #[error("Immutable selector cannot be modified")]
    ImmutableSelector = 6007,

    /// Admin capacity exceeded (max 10)
    #[error("Admin capacity exceeded")]
    AdminCapacityExceeded = 6008,

    /// Admin not found
    #[error("Admin not found")]
    AdminNotFound = 6009,

    /// Invalid PDA derivation
    #[error("Invalid PDA")]
    InvalidPDA = 6010,
}

impl From<DiamondError> for ProgramError {
    fn from(e: DiamondError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for DiamondError {
    fn type_of() -> &'static str {
        "DiamondError"
    }
}

impl PrintProgramError for DiamondError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(DiamondError::ModuleNotFound as u32, 6000);
        assert_eq!(DiamondError::UnauthorizedAccess as u32, 6001);
    }

    #[test]
    fn test_error_conversion() {
        let err: ProgramError = DiamondError::ModuleNotFound.into();
        match err {
            ProgramError::Custom(code) => assert_eq!(code, 6000),
            _ => panic!("Wrong error type"),
        }
    }
}
