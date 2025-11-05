/*!
 * Diamond State Module
 * Core state management and access control
 */

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    system_program,
    sysvar::Sysvar,
};

use crate::error::DiamondError;

/// Selector mapping: 4-byte selector → program ID
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct SelectorMapping {
    pub selector: [u8; 4],
    pub module: Pubkey,
    pub function_name: [u8; 64],
    pub is_immutable: bool,
}

impl SelectorMapping {
    pub fn new(selector: [u8; 4], module: Pubkey, name: &str, immutable: bool) -> Self {
        let mut function_name = [0u8; 64];
        let bytes = name.as_bytes();
        let len = bytes.len().min(64);
        function_name[..len].copy_from_slice(&bytes[..len]);
        
        Self {
            selector,
            module,
            function_name,
            is_immutable: immutable,
        }
    }
}

/// Module metadata
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct ModuleMeta {
    pub name: [u8; 32],
    pub address: Pubkey,
    pub version: u16,
    pub is_active: bool,
}

impl ModuleMeta {
    pub fn new(name: &str, address: Pubkey, version: u16) -> Self {
        let mut name_bytes = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(32);
        name_bytes[..len].copy_from_slice(&bytes[..len]);
        
        Self {
            name: name_bytes,
            address,
            version,
            is_active: true,
        }
    }
}

/// Main Diamond State
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct DiamondState {
    pub owner: Pubkey,
    pub admins: Vec<Pubkey>,
    pub active_modules: Vec<ModuleMeta>,
    pub selectors: Vec<SelectorMapping>,
    pub bump: u8,
    pub is_paused: bool,
    pub pause_authority: Pubkey,
}

impl DiamondState {
    pub const MAX_ADMINS: usize = 10;
    pub const MAX_MODULES: usize = 20;
    pub const MAX_SELECTORS: usize = 50;
    
    pub const SPACE: usize = 
        8 +  // discriminator
        32 + // owner
        4 + (Self::MAX_ADMINS * 32) + // admins vec
        4 + (Self::MAX_MODULES * 67) + // modules vec (32 name + 32 address + 2 version + 1 is_active)
        4 + (Self::MAX_SELECTORS * 101) + // selectors vec (4 selector + 32 module + 64 function_name + 1 is_immutable)
        1 +  // bump
        1 +  // is_paused
        32;  // pause_authority
    
    pub fn new(owner: Pubkey, bump: u8) -> Self {
        Self {
            owner,
            admins: Vec::new(),
            active_modules: Vec::new(),
            selectors: Vec::new(),
            bump,
            is_paused: false,
            pause_authority: owner,
        }
    }
    
    pub fn get_module_by_selector(&self, selector: [u8; 4]) -> Option<Pubkey> {
        self.selectors
            .iter()
            .find(|s| s.selector == selector)
            .map(|s| s.module)
    }
    
    pub fn is_owner(&self, pubkey: &Pubkey) -> bool {
        &self.owner == pubkey
    }
    
    pub fn is_admin(&self, pubkey: &Pubkey) -> bool {
        self.admins.contains(pubkey)
    }
    
    pub fn has_authority(&self, pubkey: &Pubkey) -> bool {
        self.is_owner(pubkey) || self.is_admin(pubkey)
    }
}

/// Initialize diamond state
pub fn initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let diamond_state_account = next_account_info(account_iter)?;
    let owner_account = next_account_info(account_iter)?;
    let payer = next_account_info(account_iter)?;
    let system_program_account = next_account_info(account_iter)?;
    
    // Validate
    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if system_program_account.key != &system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Parse data
    #[derive(BorshDeserialize)]
    struct InitData {
        owner: Pubkey,
        bump: u8,
    }
    
    let init_data = InitData::try_from_slice(data)?;
    
    // Verify PDA
    let (pda, expected_bump) = Pubkey::find_program_address(
        &[b"diamond_state", init_data.owner.as_ref()],
        program_id,
    );
    
    if diamond_state_account.key != &pda {
        return Err(ProgramError::InvalidSeeds);
    }
    if init_data.bump != expected_bump {
        return Err(ProgramError::InvalidSeeds);
    }
    
    // Create account
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(DiamondState::SPACE);
    
    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            diamond_state_account.key,
            lamports,
            DiamondState::SPACE as u64,
            program_id,
        ),
        &[payer.clone(), diamond_state_account.clone(), system_program_account.clone()],
        &[&[b"diamond_state", init_data.owner.as_ref(), &[init_data.bump]]],
    )?;
    
    // Initialize state
    let state = DiamondState::new(init_data.owner, init_data.bump);
    state.serialize(&mut &mut diamond_state_account.data.borrow_mut()[..])?;
    
    msg!("Diamond initialized for owner: {}", init_data.owner);
    Ok(())
}

/// Add admin
pub fn add_admin(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let state_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let new_admin = Pubkey::try_from_slice(data)?;
    
    let mut state = DiamondState::try_from_slice(&state_account.data.borrow())?;
    
    if !state.is_owner(authority.key) {
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    if state.admins.len() >= DiamondState::MAX_ADMINS {
        return Err(DiamondError::AdminCapacityExceeded.into());
    }
    
    if !state.admins.contains(&new_admin) {
        state.admins.push(new_admin);
    }
    
    state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
    
    msg!("Admin added: {}", new_admin);
    Ok(())
}

/// Pause/unpause diamond
pub fn pause(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let state_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let should_pause = bool::try_from_slice(data)?;
    
    let mut state = DiamondState::try_from_slice(&state_account.data.borrow())?;
    
    if !state.has_authority(authority.key) {
        return Err(DiamondError::UnauthorizedAccess.into());
    }
    
    state.is_paused = should_pause;
    state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
    
    msg!("Diamond paused: {}", should_pause);
    Ok(())
}
