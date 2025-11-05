/*!
 * Diamond Error Types
 * Native Rust error handling
 */

use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[repr(u32)]
pub enum DiamondError {
    #[error("Module not found")]
    ModuleNotFound = 6000,
    
    #[error("Unauthorized access")]
    UnauthorizedAccess = 6001,
    
    #[error("Invalid selector")]
    InvalidSelector = 6002,
    
    #[error("Module capacity exceeded")]
    ModuleCapacityExceeded = 6003,
    
    #[error("Selector capacity exceeded")]
    SelectorCapacityExceeded = 6004,
    
    #[error("Selector collision")]
    SelectorCollision = 6005,
    
    #[error("Diamond is paused")]
    DiamondPaused = 6006,
    
    #[error("Immutable selector")]
    ImmutableSelector = 6007,
    
    #[error("Admin capacity exceeded")]
    AdminCapacityExceeded = 6008,
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
