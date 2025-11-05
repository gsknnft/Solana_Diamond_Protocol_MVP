/*!
 * Example Native Facet - Counter
 * 
 * Demonstrates a simple facet that can be called via the diamond router.
 * This is a complete, working example showing:
 * - Selector-based routing
 * - State management
 * - Pure Rust implementation
 */

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Program ID (placeholder - replace with actual deployed program ID)
solana_program::declare_id!("FacetNativeExamp1e1111111111111111111111111");

// Function selectors (would be registered in diamond router)
pub const INCREMENT_SELECTOR: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
pub const DECREMENT_SELECTOR: [u8; 4] = [0x05, 0x06, 0x07, 0x08];
pub const GET_VALUE_SELECTOR: [u8; 4] = [0x09, 0x0A, 0x0B, 0x0C];
pub const RESET_SELECTOR: [u8; 4] = [0x0D, 0x0E, 0x0F, 0x10];

/// Counter state
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Counter {
    pub value: u64,
    pub authority: Pubkey,
    pub bump: u8,
}

impl Counter {
    pub const SPACE: usize = 8 + 8 + 32 + 1; // discriminator + value + authority + bump
    
    pub fn new(authority: Pubkey, bump: u8) -> Self {
        Self {
            value: 0,
            authority,
            bump,
        }
    }
}

// Program entrypoint
entrypoint!(process_instruction);

/// Main instruction processor
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Example Facet: Processing instruction");
    
    // Validate instruction has selector
    if instruction_data.len() < 4 {
        msg!("Error: Instruction data too short");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // Extract selector
    let selector: [u8; 4] = instruction_data[..4]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    let data = &instruction_data[4..];
    
    // Route to handler
    match selector {
        INCREMENT_SELECTOR => {
            msg!("Function: Increment");
            increment(accounts, data)
        }
        DECREMENT_SELECTOR => {
            msg!("Function: Decrement");
            decrement(accounts, data)
        }
        GET_VALUE_SELECTOR => {
            msg!("Function: GetValue");
            get_value(accounts)
        }
        RESET_SELECTOR => {
            msg!("Function: Reset");
            reset(accounts)
        }
        _ => {
            msg!("Error: Unknown selector: {:?}", selector);
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

/// Increment the counter
fn increment(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let counter_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Validate authority
    if !authority.is_signer {
        msg!("Error: Authority must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Load counter
    let mut counter_data = counter_account.try_borrow_mut_data()?;
    let mut counter = Counter::try_from_slice(&counter_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Verify authority
    if counter.authority != *authority.key {
        msg!("Error: Invalid authority");
        return Err(ProgramError::IllegalOwner);
    }
    
    // Parse amount (optional, defaults to 1)
    let amount = if data.is_empty() {
        1u64
    } else {
        match u64::try_from_slice(data) {
            Ok(val) => val,
            Err(e) => {
                msg!("Warning: Failed to parse amount ({}), using default 1", e);
                1
            }
        }
    };
    
    // Increment
    counter.value = counter
        .value
        .checked_add(amount)
        .ok_or(ProgramError::InvalidArgument)?;
    
    // Save
    counter.serialize(&mut &mut counter_data[..])?;
    
    msg!("Counter incremented by {} to {}", amount, counter.value);
    Ok(())
}

/// Decrement the counter
fn decrement(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let counter_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Validate authority
    if !authority.is_signer {
        msg!("Error: Authority must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Load counter
    let mut counter_data = counter_account.try_borrow_mut_data()?;
    let mut counter = Counter::try_from_slice(&counter_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Verify authority
    if counter.authority != *authority.key {
        msg!("Error: Invalid authority");
        return Err(ProgramError::IllegalOwner);
    }
    
    // Parse amount (optional, defaults to 1)
    let amount = if data.is_empty() {
        1u64
    } else {
        match u64::try_from_slice(data) {
            Ok(val) => val,
            Err(e) => {
                msg!("Warning: Failed to parse amount ({}), using default 1", e);
                1
            }
        }
    };
    
    // Decrement
    counter.value = counter
        .value
        .checked_sub(amount)
        .ok_or(ProgramError::InvalidArgument)?;
    
    // Save
    counter.serialize(&mut &mut counter_data[..])?;
    
    msg!("Counter decremented by {} to {}", amount, counter.value);
    Ok(())
}

/// Get counter value (read-only)
fn get_value(accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let counter_account = next_account_info(account_iter)?;
    
    // Load counter
    let counter_data = counter_account.try_borrow_data()?;
    let counter = Counter::try_from_slice(&counter_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    msg!("Counter value: {}", counter.value);
    msg!("Authority: {}", counter.authority);
    
    Ok(())
}

/// Reset counter to zero
fn reset(accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let counter_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Validate authority
    if !authority.is_signer {
        msg!("Error: Authority must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Load counter
    let mut counter_data = counter_account.try_borrow_mut_data()?;
    let mut counter = Counter::try_from_slice(&counter_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Verify authority
    if counter.authority != *authority.key {
        msg!("Error: Invalid authority");
        return Err(ProgramError::IllegalOwner);
    }
    
    // Reset
    counter.value = 0;
    
    // Save
    counter.serialize(&mut &mut counter_data[..])?;
    
    msg!("Counter reset to 0");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selectors_unique() {
        assert_ne!(INCREMENT_SELECTOR, DECREMENT_SELECTOR);
        assert_ne!(INCREMENT_SELECTOR, GET_VALUE_SELECTOR);
        assert_ne!(DECREMENT_SELECTOR, GET_VALUE_SELECTOR);
    }
    
    #[test]
    fn test_counter_creation() {
        let authority = Pubkey::default();
        let counter = Counter::new(authority, 255);
        
        assert_eq!(counter.value, 0);
        assert_eq!(counter.authority, authority);
        assert_eq!(counter.bump, 255);
    }
    
    #[test]
    fn test_counter_serialization() {
        let authority = Pubkey::default();
        let counter = Counter::new(authority, 255);
        
        let mut buffer = Vec::new();
        counter.serialize(&mut buffer).unwrap();
        
        let deserialized = Counter::try_from_slice(&buffer).unwrap();
        assert_eq!(deserialized.value, counter.value);
        assert_eq!(deserialized.authority, counter.authority);
    }
}
