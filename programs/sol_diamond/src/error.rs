use anchor_lang::prelude::*;

#[error_code]
pub enum DiamondError {
    #[msg("Facet not found for selector")]
    FacetNotFound,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Selector already registered")]
    SelectorCollision,
    
    #[msg("Cannot remove immutable facet")]
    ImmutableFacet,
    
    #[msg("Diamond is paused")]
    DiamondPaused,
    
    #[msg("Maximum facets reached")]
    MaxFacetsReached,
}
