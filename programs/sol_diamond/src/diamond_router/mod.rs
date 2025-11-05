/*!
 * Diamond Router Module
 * Core dispatch logic - forwards calls to facets
 */

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};
use crate::error::DiamondError;

/// Dispatch instruction to registered facet
pub fn dispatch(ctx: Context<crate::Dispatch>, ix_data: Vec<u8>) -> Result<()> {
    let diamond = &ctx.accounts.diamond_state;
    let facet_program = &ctx.accounts.facet_program;
    
    // Check if paused
    require!(!diamond.is_paused, DiamondError::DiamondPaused);
    
    // Extract selector (first 4 bytes)
    require!(ix_data.len() >= 4, DiamondError::FacetNotFound);
    let selector: [u8; 4] = ix_data[..4].try_into().unwrap();
    
    msg!("Dispatching with selector: {:?}", selector);
    
    // Lookup facet
    let expected_facet = diamond
        .get_facet_by_selector(selector)
        .ok_or(DiamondError::FacetNotFound)?;
    
    // Validate provided facet matches registry
    require!(
        facet_program.key() == expected_facet,
        DiamondError::Unauthorized
    );
    
    msg!("Forwarding to facet: {}", expected_facet);
    
    // Forward via CPI
    let ix = Instruction {
        program_id: *facet_program.key,
        accounts: ctx.remaining_accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect(),
        data: ix_data,
    };
    
    invoke(&ix, ctx.remaining_accounts)?;
    
    msg!("Dispatch successful");
    Ok(())
}
