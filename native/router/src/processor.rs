/*!
 * Instruction Processors - Native Rust Implementation
 * 
 * Core dispatch logic identical to Anchor version,
 * but with manual account parsing and validation.
 */

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    system_program,
    instruction::{AccountMeta, Instruction},
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::DiamondError,
    state::{DiamondState, ModuleMeta, SelectorMapping},
};

/// Initialize diamond state account
pub fn initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Processing: Initialize");
    
    // Parse accounts
    let account_iter = &mut accounts.iter();
    let diamond_state_account = next_account_info(account_iter)?;
    let owner_account = next_account_info(account_iter)?;
    let payer = next_account_info(account_iter)?;
    let system_program_account = next_account_info(account_iter)?;
    
    // Validate accounts
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    if system_program_account.key != &system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Parse instruction data (owner pubkey + bump)
    #[derive(BorshDeserialize)]
    struct InitializeData {
        owner: Pubkey,
        bump: u8,
    }
    
    let init_data = InitializeData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Verify PDA derivation
    let (expected_pda, expected_bump) = Pubkey::find_program_address(
        &[b"diamond_state", init_data.owner.as_ref()],
        program_id,
    );
    
    if diamond_state_account.key != &expected_pda {
        msg!("Error: Invalid PDA derivation");
        return Err(ProgramError::InvalidSeeds);
    }
    
    if init_data.bump != expected_bump {
        msg!("Error: Invalid bump seed");
        return Err(ProgramError::InvalidSeeds);
    }
    
    // Create account via CPI
    let rent = Rent::get()?;
    let space = DiamondState::SPACE;
    let lamports = rent.minimum_balance(space);
    
    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            diamond_state_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            payer.clone(),
            diamond_state_account.clone(),
            system_program_account.clone(),
        ],
        &[&[b"diamond_state", init_data.owner.as_ref(), &[init_data.bump]]],
    )?;
    
    // Initialize state
    let diamond_state = DiamondState::new(init_data.owner, init_data.bump);
    
    // Serialize to account
    let mut data = diamond_state_account.try_borrow_mut_data()?;
    borsh::to_writer(&mut data[..], &diamond_state)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    msg!("Diamond initialized successfully");
    Ok(())
}

/// Dispatch instruction to registered facet (CORE LOGIC)
pub fn dispatch(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Processing: Dispatch");
    
    // Parse accounts
    let account_iter = &mut accounts.iter();
    let router_config_account = next_account_info(account_iter)?;
    let module_account = next_account_info(account_iter)?;
    let remaining_accounts = account_iter.as_slice();
    
    // Validate router config is writable
    if !router_config_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Deserialize diamond state
    let router_config_data = router_config_account.try_borrow_data()?;
    let router_config = DiamondState::try_from_slice(&router_config_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Check if paused
    if router_config.is_paused {
        msg!("Error: Diamond is paused");
        return Err(DiamondError::DiamondPaused.into());
    }
    
    // Parse instruction data (ix_data as Vec<u8>)
    let ix_data = Vec::<u8>::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    if ix_data.len() < 4 {
        msg!("Error: Instruction data too short (need at least 4 bytes for selector)");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // Extract selector (first 4 bytes)
    let selector: [u8; 4] = ix_data[..4].try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    msg!("Selector: {:?}", selector);
    
    // Lookup facet program by selector (CORE DISPATCH LOGIC)
    let expected_program = router_config.get_module_by_selector(selector)
        .ok_or_else(|| {
            msg!("Error: Module not found for selector {:?}", selector);
            DiamondError::ModuleNotFound
        })?;
    
    msg!("Target module: {}", expected_program);
    
    // Validate passed module matches registry
    if module_account.key != &expected_program {
        msg!("Error: Module mismatch. Expected: {}, Got: {}", expected_program, module_account.key);
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    // Forward instruction via CPI
    msg!("Forwarding to facet via CPI");
    
    let ix = Instruction {
        program_id: *module_account.key,
        accounts: remaining_accounts.iter()
            .map(|account| AccountMeta {
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

/// Add a new module (facet) to the diamond
pub fn add_module(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Processing: AddModule");
    
    // Parse accounts
    let account_iter = &mut accounts.iter();
    let diamond_state_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Validate authority is signer
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse instruction data
    #[derive(BorshDeserialize)]
    struct AddModuleData {
        module_address: Pubkey,
        selector: [u8; 4],
    }
    
    let add_data = AddModuleData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Deserialize, modify, and reserialize state
    let mut state_data = diamond_state_account.try_borrow_mut_data()?;
    let mut state = DiamondState::try_from_slice(&state_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Check authority
    if !state.is_owner(authority.key) {
        msg!("Error: Only owner can add modules");
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    // Add module metadata
    let module_meta = ModuleMeta::new("new_module", add_data.module_address, 1);
    state.add_module(module_meta)
        .map_err(|_| DiamondError::ModuleCapacityExceeded)?;
    
    // Add selector mapping
    let selector_mapping = SelectorMapping::new(
        add_data.selector,
        add_data.module_address,
        "function",
        false,
    );
    state.add_selector(selector_mapping)
        .map_err(|_| DiamondError::SelectorCapacityExceeded)?;
    
    // Serialize back
    borsh::to_writer(&mut state_data[..], &state)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    msg!("Module added: {}, Selector: {:?}", add_data.module_address, add_data.selector);
    Ok(())
}

/// Remove a module from the diamond
pub fn remove_module(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Processing: RemoveModule");
    
    // Parse accounts
    let account_iter = &mut accounts.iter();
    let diamond_state_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Validate authority is signer
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse instruction data (selector to remove)
    #[derive(BorshDeserialize)]
    struct RemoveModuleData {
        selector: [u8; 4],
    }
    
    let remove_data = RemoveModuleData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Deserialize, modify, and reserialize state
    let mut state_data = diamond_state_account.try_borrow_mut_data()?;
    let mut state = DiamondState::try_from_slice(&state_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Check authority
    if !state.is_owner(authority.key) {
        msg!("Error: Only owner can remove modules");
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    // Check if selector is immutable
    if let Some(mapping) = state.selectors.iter().find(|s| s.selector == remove_data.selector) {
        if mapping.is_immutable {
            msg!("Error: Cannot remove immutable selector");
            return Err(DiamondError::ImmutableSelector.into());
        }
    }
    
    // Remove selector
    state.selectors.retain(|s| s.selector != remove_data.selector);
    
    // Serialize back
    borsh::to_writer(&mut state_data[..], &state)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    msg!("Module removed for selector: {:?}", remove_data.selector);
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
