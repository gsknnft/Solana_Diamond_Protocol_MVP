/*!
 * Solana Diamond Protocol - Minimal MVP
 * 
 * A minimal, production-ready implementation of the diamond pattern on Solana.
 * This MVP demonstrates the core functionality without advanced features.
 */

use anchor_lang::prelude::*;

pub mod diamond_state;
pub mod diamond_router;
pub mod diamond_cut;
pub mod error;

declare_id!("DiamondMVP1111111111111111111111111111111");

// Re-export main types
pub use diamond_state::{DiamondState, ModuleMeta, SelectorMapping};
pub use error::DiamondError;

#[program]
pub mod sol_diamond_mvp {
    use super::*;

    /// Initialize the diamond with an owner
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        diamond_state::initialize(ctx)
    }

    /// Dispatch instruction to a registered facet
    pub fn dispatch(ctx: Context<Dispatch>, ix_data: Vec<u8>) -> Result<()> {
        diamond_router::dispatch(ctx, ix_data)
    }

    /// Add a new facet to the diamond
    pub fn add_facet(
        ctx: Context<AddFacet>,
        selector: [u8; 4],
        module_address: Pubkey,
        function_name: String,
        is_immutable: bool,
    ) -> Result<()> {
        diamond_cut::add_facet(ctx, selector, module_address, function_name, is_immutable)
    }

    /// Remove a facet from the diamond
    pub fn remove_facet(ctx: Context<RemoveFacet>, selector: [u8; 4]) -> Result<()> {
        diamond_cut::remove_facet(ctx, selector)
    }

    /// Pause/unpause the diamond
    pub fn set_paused(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
        diamond_state::set_paused(ctx, paused)
    }
}

// ===== Context Structs =====

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = DiamondState::SPACE,
        seeds = [b"diamond_state", owner.key().as_ref()],
        bump
    )]
    pub diamond_state: Account<'info, DiamondState>,
    
    /// CHECK: The owner of the diamond
    pub owner: AccountInfo<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Dispatch<'info> {
    #[account(mut)]
    pub diamond_state: Account<'info, DiamondState>,
    
    /// CHECK: Validated in instruction
    pub facet_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AddFacet<'info> {
    #[account(
        mut,
        has_one = owner @ DiamondError::Unauthorized
    )]
    pub diamond_state: Account<'info, DiamondState>,
    
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveFacet<'info> {
    #[account(
        mut,
        has_one = owner @ DiamondError::Unauthorized
    )]
    pub diamond_state: Account<'info, DiamondState>,
    
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPaused<'info> {
    #[account(
        mut,
        has_one = owner @ DiamondError::Unauthorized
    )]
    pub diamond_state: Account<'info, DiamondState>,
    
    pub owner: Signer<'info>,
}
