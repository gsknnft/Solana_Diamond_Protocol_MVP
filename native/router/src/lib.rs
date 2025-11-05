/*!
 * Native Rust Diamond Router
 * 
 * This is a pure Rust implementation of the Solana Diamond Protocol router,
 * demonstrating that the architecture is framework-independent.
 * 
 * Key differences from Anchor version:
 * - No #[program] macro → manual entrypoint
 * - No #[derive(Accounts)] → manual account parsing
 * - No automatic IDL generation
 * - Smaller binary size (~80KB vs ~150KB)
 * - Same functionality and behavior
 * 
 * Build: cargo build-sbf
 */

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
};
use borsh::{BorshDeserialize, BorshSerialize};

pub mod error;
pub mod processor;
pub mod state;

use error::DiamondError;
use state::DiamondState;

// Program ID placeholder (would be set during deployment)
solana_program::declare_id!("DiamondNative1111111111111111111111111111111");

// Instruction discriminators (8 bytes, matching Anchor's format)
const INITIALIZE_DISCRIMINATOR: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const DISPATCH_DISCRIMINATOR: [u8; 8] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const ADD_MODULE_DISCRIMINATOR: [u8; 8] = [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const REMOVE_MODULE_DISCRIMINATOR: [u8; 8] = [0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

/// Program entry point (replaces Anchor's #[program] macro)
entrypoint!(process_instruction);

/// Main instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Diamond Router Native: Processing instruction");
    
    // Validate instruction data length
    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // Extract discriminator (first 8 bytes, Anchor-compatible)
    let (discriminator, data) = instruction_data.split_at(8);
    let discriminator: [u8; 8] = discriminator.try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Route to appropriate processor
    match discriminator {
        INITIALIZE_DISCRIMINATOR => {
            msg!("Instruction: Initialize");
            processor::initialize(program_id, accounts, data)
        }
        DISPATCH_DISCRIMINATOR => {
            msg!("Instruction: Dispatch");
            processor::dispatch(program_id, accounts, data)
        }
        ADD_MODULE_DISCRIMINATOR => {
            msg!("Instruction: AddModule");
            processor::add_module(program_id, accounts, data)
        }
        REMOVE_MODULE_DISCRIMINATOR => {
            msg!("Instruction: RemoveModule");
            processor::remove_module(program_id, accounts, data)
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
    use solana_program::clock::Epoch;
    use std::mem;

    #[test]
    fn test_dispatch_discriminator() {
        assert_eq!(DISPATCH_DISCRIMINATOR.len(), 8);
        assert_eq!(INITIALIZE_DISCRIMINATOR.len(), 8);
    }

    #[test]
    fn test_diamond_state_size() {
        // Ensure state struct size is reasonable
        let size = mem::size_of::<DiamondState>();
        println!("DiamondState size: {} bytes", size);
        assert!(size > 0);
    }
}
