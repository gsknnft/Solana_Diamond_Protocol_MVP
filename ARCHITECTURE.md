# Architecture Overview

## Core Concept

The Diamond Protocol implements **modular smart contracts** on Solana by using a central router that dispatches calls to separate "facet" programs.

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ Instruction + Selector
       ▼
┌─────────────────────┐
│  Diamond Router     │
│  ┌───────────────┐  │
│  │ 1. Extract    │  │
│  │    Selector   │  │
│  │ 2. Lookup     │  │
│  │    Facet      │  │
│  │ 3. Validate   │  │
│  │ 4. CPI Forward│  │
│  └───────────────┘  │
└──────┬──────────────┘
       │
       ├─────CPI────→ Facet A
       ├─────CPI────→ Facet B
       └─────CPI────→ Facet C
```

## Components

### 1. Diamond Router

**Purpose**: Central dispatcher that routes instructions to facets

**State**:
- `owner` - Authority who can modify the diamond
- `selectors` - Map of `[u8; 4]` selectors to facet program IDs
- `modules` - Metadata about registered facets
- `is_paused` - Emergency pause flag

**Instructions**:
- `initialize` - Create the diamond
- `dispatch` - Route instruction to facet
- `add_facet` - Register new functionality
- `remove_facet` - Remove functionality
- `set_paused` - Pause/unpause

### 2. Facets

**Purpose**: Independent programs implementing specific functionality

**Requirements**:
- Accept instruction with 4-byte selector
- Implement business logic
- Return success/error

**Example**: Counter facet
```rust
pub const INCREMENT: [u8; 4] = [0x01, 0x02, 0x03, 0x04];

pub fn process_instruction(...) {
    let selector = instruction_data[..4];
    match selector {
        INCREMENT => increment(),
        // ...
    }
}
```

### 3. Selector Registry

**Purpose**: Map function selectors to facet programs

```rust
pub struct SelectorMapping {
    pub selector: [u8; 4],      // Function identifier
    pub module: Pubkey,          // Facet program ID
    pub function_name: String,   // Human-readable name
    pub is_immutable: bool,      // Can be removed?
}
```

## Dispatch Flow

### Step-by-Step

1. **Client sends instruction**:
   ```
   Instruction {
       program_id: diamond_router,
       data: [selector (4 bytes), ...args]
   }
   ```

2. **Router extracts selector**:
   ```rust
   let selector: [u8; 4] = ix_data[..4].try_into()?;
   ```

3. **Router lookups facet**:
   ```rust
   let facet = state.get_facet_by_selector(selector)?;
   ```

4. **Router validates**:
   ```rust
   require!(
       provided_facet.key() == facet,
       "Unauthorized"
   );
   ```

5. **Router forwards via CPI**:
   ```rust
   invoke(&Instruction {
       program_id: facet,
       accounts: remaining_accounts,
       data: ix_data,
   }, remaining_accounts)?;
   ```

## Design Decisions

### Why 4-byte Selectors?

- **Standard**: Matches Ethereum's function selectors
- **Compact**: Small instruction overhead
- **Sufficient**: 4 billion possible selectors
- **EIP-2535 Compatible**: Follows diamond standard

### Why Separate Programs?

Solana's limitations make this necessary:
- **10MB program size limit** - Split logic across programs
- **Independent upgrades** - Update facets without touching router
- **Code reuse** - Share facets across multiple diamonds

### Why PDA for State?

- **Deterministic addresses** - `find_program_address(["diamond_state", owner])`
- **No keypair management** - Derived from seeds
- **Program-owned** - Router controls the state

### Why CPI?

- **Native Solana** - Built-in cross-program invocation
- **Account forwarding** - Pass all accounts through
- **Atomic execution** - All or nothing
- **Gas efficiency** - No external messaging needed

## Security Model

### Access Control

1. **Owner-only operations**:
   - Add facets
   - Remove facets
   - Pause diamond

2. **Immutable facets**:
   - Can't be removed once added
   - For critical functionality

3. **Pause mechanism**:
   - Emergency stop
   - Blocks all dispatches

### Validation

1. **Selector lookup**:
   - Ensures facet is registered
   - Prevents unauthorized programs

2. **Facet validation**:
   - Confirms provided facet matches registry
   - Prevents facet impersonation

3. **Account validation**:
   - Each facet validates its accounts
   - Router doesn't validate facet logic

## Comparison: Anchor vs Native

### Anchor Version

**Pros**:
- ✅ Rapid development
- ✅ Automatic IDL generation
- ✅ Type-safe contexts
- ✅ Better tooling

**Cons**:
- ❌ Larger binary (~150KB)
- ❌ Framework dependency
- ❌ Less control over serialization

### Native Version

**Pros**:
- ✅ Smaller binary (~80KB)
- ✅ No framework dependency
- ✅ Full control
- ✅ Portable to other chains

**Cons**:
- ❌ More boilerplate
- ❌ Manual account parsing
- ❌ No automatic IDL
- ❌ Steeper learning curve

**Both use identical core logic!**

## Extension Points

### Add New Facets

```rust
// Register new functionality
diamond.add_facet(
    selector: [0xAA, 0xBB, 0xCC, 0xDD],
    program: new_facet_program_id,
    name: "my_function",
    immutable: false,
);
```

### Upgrade Facets

```rust
// Deploy new version
solana program deploy new_facet.so --program-id facet_id

// No need to update router - same selector!
```

### Remove Facets

```rust
// Remove mutable facets
diamond.remove_facet(selector);
```

## Best Practices

1. **Plan selectors carefully** - They're your API
2. **Use immutable for core functions** - Can't be removed
3. **Implement pause** - For emergency situations
4. **Version facets** - Track what's deployed
5. **Test extensively** - CPI adds complexity
6. **Document selectors** - Make them discoverable

## Further Reading

- [EIP-2535 Diamond Standard](https://eips.ethereum.org/EIPS/eip-2535)
- [Solana CPI Documentation](https://docs.solana.com/developing/programming-model/calling-between-programs)
- [Anchor Book](https://book.anchor-lang.com/)

---

**Questions?** See the code in `programs/sol_diamond/` or `native/router/`
