/*!
 * Diamond State Module
 * Core state management for the diamond
 */

use anchor_lang::prelude::*;
use crate::error::DiamondError;

/// Selector mapping: function selector -> facet program
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SelectorMapping {
    pub selector: [u8; 4],
    pub module: Pubkey,
    pub function_name: String,
    pub is_immutable: bool,
}

/// Facet metadata
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ModuleMeta {
    pub name: String,
    pub address: Pubkey,
    pub version: u16,
}

/// Main Diamond State Account
#[account]
pub struct DiamondState {
    pub owner: Pubkey,
    pub selectors: Vec<SelectorMapping>,
    pub modules: Vec<ModuleMeta>,
    pub bump: u8,
    pub is_paused: bool,
}

impl DiamondState {
    pub const MAX_SELECTORS: usize = 50;
    pub const MAX_MODULES: usize = 20;
    
    pub const SPACE: usize = 8 + // discriminator
        32 + // owner
        4 + (Self::MAX_SELECTORS * 150) + // selectors (generous estimate)
        4 + (Self::MAX_MODULES * 100) + // modules
        1 + // bump
        1; // is_paused
    
    pub fn get_facet_by_selector(&self, selector: [u8; 4]) -> Option<Pubkey> {
        self.selectors
            .iter()
            .find(|s| s.selector == selector)
            .map(|s| s.module)
    }
}

/// Initialize the diamond
pub fn initialize(ctx: Context<crate::Initialize>) -> Result<()> {
    let diamond = &mut ctx.accounts.diamond_state;
    
    diamond.owner = ctx.accounts.owner.key();
    diamond.selectors = Vec::new();
    diamond.modules = Vec::new();
    diamond.bump = ctx.bumps.diamond_state;
    diamond.is_paused = false;
    
    msg!("Diamond initialized with owner: {}", diamond.owner);
    Ok(())
}

/// Set paused state
pub fn set_paused(ctx: Context<crate::SetPaused>, paused: bool) -> Result<()> {
    let diamond = &mut ctx.accounts.diamond_state;
    diamond.is_paused = paused;
    
    msg!("Diamond paused state set to: {}", paused);
    Ok(())
}
