/*!
 * Diamond Cut Module
 * Add/remove facets dynamically
 */

use anchor_lang::prelude::*;
use crate::diamond_state::{SelectorMapping, ModuleMeta};
use crate::error::DiamondError;

/// Add a new facet to the diamond
pub fn add_facet(
    ctx: Context<crate::AddFacet>,
    selector: [u8; 4],
    module_address: Pubkey,
    function_name: String,
    is_immutable: bool,
) -> Result<()> {
    let diamond = &mut ctx.accounts.diamond_state;
    
    // Check capacity
    require!(
        diamond.selectors.len() < crate::diamond_state::DiamondState::MAX_SELECTORS,
        DiamondError::MaxFacetsReached
    );
    
    // Check for collision
    require!(
        diamond.get_facet_by_selector(selector).is_none(),
        DiamondError::SelectorCollision
    );
    
    // Add selector mapping
    diamond.selectors.push(SelectorMapping {
        selector,
        module: module_address,
        function_name: function_name.clone(),
        is_immutable,
    });
    
    // Add module if not already present
    if !diamond.modules.iter().any(|m| m.address == module_address) {
        diamond.modules.push(ModuleMeta {
            name: function_name.clone(),
            address: module_address,
            version: 1,
        });
    }
    
    msg!(
        "Facet added: selector {:?} -> {} ({})",
        selector,
        module_address,
        function_name
    );
    
    Ok(())
}

/// Remove a facet from the diamond
pub fn remove_facet(ctx: Context<crate::RemoveFacet>, selector: [u8; 4]) -> Result<()> {
    let diamond = &mut ctx.accounts.diamond_state;
    
    // Find selector
    let index = diamond
        .selectors
        .iter()
        .position(|s| s.selector == selector)
        .ok_or(DiamondError::FacetNotFound)?;
    
    // Check if immutable
    require!(
        !diamond.selectors[index].is_immutable,
        DiamondError::ImmutableFacet
    );
    
    // Remove
    diamond.selectors.remove(index);
    
    msg!("Facet removed: selector {:?}", selector);
    Ok(())
}
