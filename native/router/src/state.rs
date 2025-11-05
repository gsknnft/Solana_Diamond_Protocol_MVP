/*!
 * Diamond State - Native Rust Implementation
 * 
 * Identical to Anchor version but without #[account] macro.
 * Uses Borsh serialization for compatibility.
 */

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Selector mapping (4-byte selector → program address)
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct SelectorMapping {
    pub namespace: [u8; 8],       // For EIP-2535 §5 compliance
    pub selector: [u8; 4],         // Function selector
    pub module: Pubkey,            // Target program ID
    pub function_name: [u8; 64],   // Human-readable name
    pub is_immutable: bool,        // EIP-2535 immutability flag
}

impl SelectorMapping {
    pub fn new(
        selector: [u8; 4],
        module: Pubkey,
        function_name: &str,
        is_immutable: bool,
    ) -> Self {
        Self::new_with_namespace([0u8; 8], selector, module, function_name, is_immutable)
    }

    pub fn new_with_namespace(
        namespace: [u8; 8],
        selector: [u8; 4],
        module: Pubkey,
        function_name: &str,
        is_immutable: bool,
    ) -> Self {
        let mut name_bytes = [0u8; 64];
        let bytes = function_name.as_bytes();
        let len = bytes.len().min(64);
        name_bytes[..len].copy_from_slice(&bytes[..len]);

        Self {
            namespace,
            selector,
            module,
            function_name: name_bytes,
            is_immutable,
        }
    }

    pub fn function_name_as_str(&self) -> &str {
        let end = self.function_name.iter()
            .position(|&c| c == 0)
            .unwrap_or(self.function_name.len());
        std::str::from_utf8(&self.function_name[..end]).unwrap_or("")
    }
}

/// Module metadata
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct ModuleMeta {
    pub name: [u8; 32],      // Fixed-size for stack safety
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

    pub fn name_as_str(&self) -> &str {
        let end = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
        std::str::from_utf8(&self.name[..end]).unwrap_or("")
    }
}

/// Main Diamond State (identical to Anchor version)
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct DiamondState {
    pub owner: Pubkey,
    pub admins: Vec<Pubkey>,
    pub active_modules: Vec<ModuleMeta>,
    pub selectors: Vec<SelectorMapping>,
    pub bump: u8,
    pub is_paused: bool,
    pub pause_authority: Pubkey,
    pub paused_at: Option<i64>,
    pub pause_reason: [u8; 64],
    pub namespaces_enabled: bool,
    pub squads_multisig: Option<Pubkey>,
    pub governance_realm: Option<Pubkey>,
    pub governance_program: Option<Pubkey>,
    pub hot_cache: [Option<SelectorMapping>; 5],
}

impl DiamondState {
    /// Maximum capacities (same as Anchor version)
    pub const MAX_ADMINS: usize = 10;
    pub const MAX_MODULES: usize = 20;
    pub const MAX_SELECTORS: usize = 50;

    /// Calculate required space for account
    pub const SPACE: usize = 
        8 +      // Discriminator (Anchor compatibility)
        32 +     // owner
        4 + (Self::MAX_ADMINS * 32) +  // admins vec
        4 + (Self::MAX_MODULES * 68) + // active_modules vec
        4 + (Self::MAX_SELECTORS * 113) + // selectors vec
        1 +      // bump
        1 +      // is_paused
        32 +     // pause_authority
        9 +      // paused_at (Option<i64>)
        64 +     // pause_reason
        1 +      // namespaces_enabled
        33 +     // squads_multisig (Option<Pubkey>)
        33 +     // governance_realm
        33 +     // governance_program
        (5 * 114); // hot_cache array

    /// Initialize new diamond state
    pub fn new(owner: Pubkey, bump: u8) -> Self {
        Self {
            owner,
            admins: Vec::new(),
            active_modules: Vec::new(),
            selectors: Vec::new(),
            bump,
            is_paused: false,
            pause_authority: owner,
            paused_at: None,
            pause_reason: [0u8; 64],
            namespaces_enabled: false,
            squads_multisig: None,
            governance_realm: None,
            governance_program: None,
            hot_cache: [None, None, None, None, None],
        }
    }

    /// Get module by selector (core dispatch logic)
    pub fn get_module_by_selector(&self, selector: [u8; 4]) -> Option<Pubkey> {
        // Check hot cache first (performance optimization)
        for cached in &self.hot_cache {
            if let Some(mapping) = cached {
                if mapping.selector == selector {
                    return Some(mapping.module);
                }
            }
        }

        // Linear search through selectors
        self.selectors.iter()
            .find(|s| s.selector == selector)
            .map(|s| s.module)
    }

    /// Add a module (validates capacity)
    pub fn add_module(&mut self, meta: ModuleMeta) -> Result<(), ProgramError> {
        if self.active_modules.len() >= Self::MAX_MODULES {
            return Err(ProgramError::Custom(1)); // Capacity exceeded
        }
        self.active_modules.push(meta);
        Ok(())
    }

    /// Add a selector mapping (validates capacity)
    pub fn add_selector(&mut self, mapping: SelectorMapping) -> Result<(), ProgramError> {
        if self.selectors.len() >= Self::MAX_SELECTORS {
            return Err(ProgramError::Custom(2)); // Capacity exceeded
        }
        
        // Check for collision
        if self.get_module_by_selector(mapping.selector).is_some() {
            return Err(ProgramError::Custom(3)); // Selector collision
        }
        
        self.selectors.push(mapping);
        Ok(())
    }

    /// Check if caller is owner
    pub fn is_owner(&self, pubkey: &Pubkey) -> bool {
        &self.owner == pubkey
    }

    /// Check if caller is admin
    pub fn is_admin(&self, pubkey: &Pubkey) -> bool {
        self.admins.contains(pubkey)
    }

    /// Check if caller has authority (owner or admin)
    pub fn has_authority(&self, pubkey: &Pubkey) -> bool {
        self.is_owner(pubkey) || self.is_admin(pubkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_mapping_creation() {
        let mapping = SelectorMapping::new(
            [0x01, 0x02, 0x03, 0x04],
            Pubkey::default(),
            "test_function",
            false,
        );
        assert_eq!(mapping.selector, [0x01, 0x02, 0x03, 0x04]);
        assert_eq!(mapping.function_name_as_str(), "test_function");
    }

    #[test]
    fn test_diamond_state_initialization() {
        let owner = Pubkey::default();
        let state = DiamondState::new(owner, 255);
        
        assert_eq!(state.owner, owner);
        assert_eq!(state.bump, 255);
        assert_eq!(state.is_paused, false);
        assert_eq!(state.selectors.len(), 0);
    }

    #[test]
    fn test_selector_lookup() {
        let owner = Pubkey::default();
        let mut state = DiamondState::new(owner, 255);
        
        let mapping = SelectorMapping::new(
            [0xAA, 0xBB, 0xCC, 0xDD],
            Pubkey::new_unique(),
            "my_function",
            false,
        );
        
        let expected_module = mapping.module;
        state.add_selector(mapping).unwrap();
        
        let found = state.get_module_by_selector([0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(found, Some(expected_module));
    }
}
