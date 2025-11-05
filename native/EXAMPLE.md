# Native Diamond Protocol - Usage Example

This document shows how to use the native (Anchor-free) diamond protocol implementation.

## Quick Start

```bash
# 1. Build everything
cd native
./build.sh

# 2. Run validator
cargo run --bin validator
```

## Architecture Overview

The native implementation has three main components:

### 1. Diamond Router (`router/`)

The central dispatcher that:
- Maintains a registry of function selectors → program addresses
- Validates incoming instructions
- Forwards calls via CPI to registered facets

**Key files:**
- `diamond_state/mod.rs` - State management & access control
- `diamond_router/mod.rs` - Core dispatch logic (CPI forwarding)
- `diamond_cut/mod.rs` - Add/remove facet modules
- `error.rs` - Custom error types

### 2. Example Facet (`facet/`)

A simple counter facet demonstrating:
- Selector-based routing (`INCREMENT`, `DECREMENT`, `GET_VALUE`, `RESET`)
- State management with Borsh serialization
- Authority checks
- Pure Rust implementation

### 3. Validator (`validator.rs`)

Simple tool that validates framework independence.

---

## Code Examples

### Dispatch Logic (Core Pattern)

The heart of the diamond protocol - identical in both Anchor and native versions:

```rust
// Extract selector from instruction data
let selector: [u8; 4] = ix_data[..4].try_into()?;

// Lookup facet program in registry
let expected_program = router_config
    .get_module_by_selector(selector)
    .ok_or(DiamondError::ModuleNotFound)?;

// Validate provided program matches registry
if module_account.key != &expected_program {
    return Err(DiamondError::UnauthorizedAccess.into());
}

// Forward via CPI
invoke(&Instruction {
    program_id: *module_account.key,
    accounts: remaining_accounts,
    data: ix_data,
}, remaining_accounts)?;
```

**This logic is framework-independent!**

### Facet Implementation

```rust
// Entry point
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Extract selector
    let selector: [u8; 4] = instruction_data[..4].try_into()?;
    
    // Route to function
    match selector {
        INCREMENT_SELECTOR => increment(accounts),
        DECREMENT_SELECTOR => decrement(accounts),
        GET_VALUE_SELECTOR => get_value(accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
```

### State Management

```rust
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct DiamondState {
    pub owner: Pubkey,
    pub admins: Vec<Pubkey>,
    pub active_modules: Vec<ModuleMeta>,
    pub selectors: Vec<SelectorMapping>,
    pub bump: u8,
    pub is_paused: bool,
}

impl DiamondState {
    pub fn get_module_by_selector(&self, selector: [u8; 4]) -> Option<Pubkey> {
        self.selectors
            .iter()
            .find(|s| s.selector == selector)
            .map(|s| s.module)
    }
}
```

---

## Deployment Example

```bash
# 1. Build for Solana BPF
cd native
cargo build-sbf

# 2. Start local validator
solana-test-validator &

# 3. Deploy router
solana program deploy \
  target/deploy/diamond_router_native.so \
  --program-id router-keypair.json

# 4. Deploy facet
solana program deploy \
  target/deploy/example_facet_native.so \
  --program-id facet-keypair.json

# 5. Initialize diamond state
# (Use TypeScript client or CLI with proper instruction data)
```

---

## Comparison: Anchor vs Native

### What's the Same

✅ **Core logic** - Dispatch, selector lookup, CPI forwarding  
✅ **State structure** - DiamondState, SelectorMapping, ModuleMeta  
✅ **PDA derivation** - `Pubkey::find_program_address()`  
✅ **Functionality** - Initialize, add/remove modules, dispatch, pause

### What's Different

| Aspect | Anchor | Native |
|--------|--------|--------|
| Entry point | `#[program]` macro | `entrypoint!()` + manual |
| Account parsing | `#[derive(Accounts)]` | Manual `next_account_info()` |
| Serialization | Automatic | Manual Borsh calls |
| Error handling | `Result<()>` | `ProgramResult` |
| IDL generation | Automatic | None |
| Build tool | `anchor build` | `cargo build-sbf` |
| Binary size | ~150KB | ~80KB |

---

## Testing

```bash
# Run unit tests
cd native
cargo test

# Run specific test
cargo test --package diamond-router-native

# With output
cargo test -- --nocapture
```

---

## Key Takeaways

1. **Framework Independence**: The diamond pattern works without Anchor
2. **Identical Logic**: Dispatch and routing are pure Rust
3. **Portability**: Can be ported to any framework or environment
4. **Production Ready**: Complete implementation with all features
5. **Educational**: Learn Solana internals without abstractions

---

## Next Steps

- Deploy to devnet/mainnet
- Add more complex facets
- Integrate with existing Anchor programs
- Port to other chains (Sui, Aptos, etc.)
- Build TypeScript SDK for native version

---

## Resources

- [Main README](README.md) - Full documentation
- [Native Rust Guide](../NATIVE_RUST_GUIDE.md) - Porting guide
- [Solana Docs](https://docs.solana.com) - Native program development

---

**Built to prove universal portability. Ready for any framework, any future.**
