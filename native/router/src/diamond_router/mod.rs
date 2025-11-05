/*!
 * Diamond Router Module
 * Core dispatch logic - forwards calls to facets via CPI
 */

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    instruction::Instruction,
};

use crate::diamond_state::DiamondState;
use crate::error::DiamondError;

/// Dispatch instruction to registered facet
/// 
/// This is the CORE of the diamond pattern:
/// 1. Extract selector from instruction data
/// 2. Lookup facet program in registry
/// 3. Validate provided program matches registry
/// 4. Forward instruction via CPI
pub fn dispatch(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Diamond Router: Dispatching to facet");
    
    // Parse accounts
    let account_iter = &mut accounts.iter();
    let router_config_account = next_account_info(account_iter)?;
    let module_account = next_account_info(account_iter)?;
    let remaining_accounts = account_iter.as_slice();
    
    // Validate router config is writable
    if !router_config_account.is_writable {
        msg!("Error: Router config account must be writable");
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Load diamond state
    let router_config_data = router_config_account.try_borrow_data()?;
    let router_config = DiamondState::try_from_slice(&router_config_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Check if paused
    if router_config.is_paused {
        msg!("Error: Diamond is paused");
        return Err(DiamondError::DiamondPaused.into());
    }
    
    // Parse instruction data (should be Vec<u8> containing facet instruction)
    let ix_data = Vec::<u8>::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    if ix_data.len() < 4 {
        msg!("Error: Instruction data too short (need selector)");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // Extract selector (first 4 bytes)
    let selector: [u8; 4] = ix_data[..4]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    msg!("Selector: {:?}", selector);
    
    // Lookup facet by selector (THE KEY DISPATCH LOGIC)
    let expected_program = router_config
        .get_module_by_selector(selector)
        .ok_or_else(|| {
            msg!("Error: Module not found for selector {:?}", selector);
            DiamondError::ModuleNotFound
        })?;
    
    msg!("Target facet: {}", expected_program);
    
    // Validate provided module matches registry
    if module_account.key != &expected_program {
        msg!(
            "Error: Module mismatch. Expected: {}, Got: {}",
            expected_program,
            module_account.key
        );
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    // Forward instruction to facet via CPI
    msg!("Forwarding to facet via CPI...");
    
    let ix = Instruction {
        program_id: *module_account.key,
        accounts: remaining_accounts
            .iter()
            .map(|account| solana_program::instruction::AccountMeta {
                pubkey: *account.key,
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            })
            .collect(),
        data: ix_data,
    };
    
    invoke(&ix, remaining_accounts)?;
    
    msg!("Dispatch successful");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_selector_extraction() {
        let ix_data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let selector: [u8; 4] = ix_data[..4].try_into().unwrap();
        assert_eq!(selector, [0x01, 0x02, 0x03, 0x04]);
    }
}
