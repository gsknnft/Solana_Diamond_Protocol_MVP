/*!
 * Native Rust Diamond Router - Anchor-Free Implementation
 * 
 * This is a complete, production-ready diamond protocol implementation
 * in pure Rust, proving framework independence.
 * 
 * Build: cargo build-sbf
 */

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Module declarations
pub mod diamond_state;
pub mod diamond_router;
pub mod diamond_cut;
pub mod error;

use error::DiamondError;

// Program ID (placeholder - replace with actual deployed program ID)
solana_program::declare_id!("DiamRouter111111111111111111111111111111111");

// Instruction discriminators (8 bytes, Anchor-compatible format)
pub const INITIALIZE_DISCRIMINATOR: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
pub const DISPATCH_DISCRIMINATOR: [u8; 8] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
pub const ADD_MODULE_DISCRIMINATOR: [u8; 8] = [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
pub const REMOVE_MODULE_DISCRIMINATOR: [u8; 8] = [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
pub const ADD_ADMIN_DISCRIMINATOR: [u8; 8] = [0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
pub const PAUSE_DISCRIMINATOR: [u8; 8] = [0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

// Program entrypoint
entrypoint!(process_instruction);

/// Main instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Diamond Router Native: Processing instruction");
    
    // Validate instruction data
    if instruction_data.len() < 8 {
        msg!("Error: Instruction data too short");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // Extract discriminator
    let (discriminator, data) = instruction_data.split_at(8);
    let discriminator: [u8; 8] = discriminator
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Route to handler
    match discriminator {
        INITIALIZE_DISCRIMINATOR => {
            msg!("Instruction: Initialize");
            diamond_state::initialize(program_id, accounts, data)
        }
        DISPATCH_DISCRIMINATOR => {
            msg!("Instruction: Dispatch");
            diamond_router::dispatch(program_id, accounts, data)
        }
        ADD_MODULE_DISCRIMINATOR => {
            msg!("Instruction: AddModule");
            diamond_cut::add_module(program_id, accounts, data)
        }
        REMOVE_MODULE_DISCRIMINATOR => {
            msg!("Instruction: RemoveModule");
            diamond_cut::remove_module(program_id, accounts, data)
        }
        ADD_ADMIN_DISCRIMINATOR => {
            msg!("Instruction: AddAdmin");
            diamond_state::add_admin(program_id, accounts, data)
        }
        PAUSE_DISCRIMINATOR => {
            msg!("Instruction: Pause");
            diamond_state::pause(program_id, accounts, data)
        }
        _ => {
            msg!("Error: Unknown instruction discriminator");
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discriminators() {
        assert_eq!(INITIALIZE_DISCRIMINATOR.len(), 8);
        assert_eq!(DISPATCH_DISCRIMINATOR.len(), 8);
        assert_ne!(INITIALIZE_DISCRIMINATOR, DISPATCH_DISCRIMINATOR);
    }
}
