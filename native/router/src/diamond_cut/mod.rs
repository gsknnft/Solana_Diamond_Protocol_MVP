/*!
 * Diamond Cut Module
 * Module (facet) management - add/remove facets
 */

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::diamond_state::{DiamondState, ModuleMeta, SelectorMapping};
use crate::error::DiamondError;

/// Add a new module (facet) to the diamond
pub fn add_module(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Diamond Cut: Adding module");
    
    let account_iter = &mut accounts.iter();
    let diamond_state_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Validate authority
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse instruction data
    #[derive(BorshDeserialize)]
    struct AddModuleData {
        module_name: String,
        module_address: Pubkey,
        selector: [u8; 4],
        function_name: String,
        is_immutable: bool,
    }
    
    let add_data = AddModuleData::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    // Load and modify state
    let mut state_data = diamond_state_account.try_borrow_mut_data()?;
    let mut state = DiamondState::try_from_slice(&state_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Check authority
    if !state.has_authority(authority.key) {
        msg!("Error: Unauthorized - only owner or admin can add modules");
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    // Check capacities
    if state.active_modules.len() >= DiamondState::MAX_MODULES {
        return Err(DiamondError::ModuleCapacityExceeded.into());
    }
    if state.selectors.len() >= DiamondState::MAX_SELECTORS {
        return Err(DiamondError::SelectorCapacityExceeded.into());
    }
    
    // Check for selector collision
    if state.get_module_by_selector(add_data.selector).is_some() {
        msg!("Error: Selector {:?} already registered", add_data.selector);
        return Err(DiamondError::SelectorCollision.into());
    }
    
    // Add module metadata
    let module_meta = ModuleMeta::new(
        &add_data.module_name,
        add_data.module_address,
        1,
    );
    state.active_modules.push(module_meta);
    
    // Add selector mapping
    let selector_mapping = SelectorMapping::new(
        add_data.selector,
        add_data.module_address,
        &add_data.function_name,
        add_data.is_immutable,
    );
    state.selectors.push(selector_mapping);
    
    // Serialize back
    state.serialize(&mut &mut state_data[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    msg!(
        "Module added: {} ({}) with selector {:?}",
        add_data.module_name,
        add_data.module_address,
        add_data.selector
    );
    Ok(())
}

/// Remove a module from the diamond
pub fn remove_module(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("Diamond Cut: Removing module");
    
    let account_iter = &mut accounts.iter();
    let diamond_state_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Validate authority
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
    
    // Load and modify state
    let mut state_data = diamond_state_account.try_borrow_mut_data()?;
    let mut state = DiamondState::try_from_slice(&state_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Check authority
    if !state.has_authority(authority.key) {
        msg!("Error: Unauthorized - only owner or admin can remove modules");
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    // Check if selector exists and is mutable
    let selector_info = state.selectors
        .iter()
        .find(|s| s.selector == remove_data.selector);
    
    match selector_info {
        None => {
            msg!("Error: Selector {:?} not found", remove_data.selector);
            return Err(DiamondError::ModuleNotFound.into());
        }
        Some(mapping) if mapping.is_immutable => {
            msg!("Error: Cannot remove immutable selector {:?}", remove_data.selector);
            return Err(DiamondError::ImmutableSelector.into());
        }
        _ => {}
    }
    
    // Remove selector
    state.selectors.retain(|s| s.selector != remove_data.selector);
    
    // Serialize back
    state.serialize(&mut &mut state_data[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    msg!("Module removed for selector: {:?}", remove_data.selector);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_module_data_serialization() {
        use borsh::BorshSerialize;
        
        #[derive(BorshSerialize, BorshDeserialize)]
        struct TestData {
            module_name: String,
            module_address: Pubkey,
            selector: [u8; 4],
            function_name: String,
            is_immutable: bool,
        }
        
        let data = TestData {
            module_name: "test_module".to_string(),
            module_address: Pubkey::default(),
            selector: [0x01, 0x02, 0x03, 0x04],
            function_name: "test_fn".to_string(),
            is_immutable: false,
        };
        
        let mut buffer = Vec::new();
        data.serialize(&mut buffer).unwrap();
        
        let deserialized = TestData::try_from_slice(&buffer).unwrap();
        assert_eq!(deserialized.module_name, "test_module");
    }
}
