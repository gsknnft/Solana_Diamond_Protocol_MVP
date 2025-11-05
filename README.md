# Solana Diamond Protocol MVP

A **minimal, production-ready** implementation of the EIP-2535 Diamond Standard for Solana, demonstrating both **Anchor** and **native Rust** approaches.

## 🎯 What's Inside

This MVP package contains two complete implementations:

### 1. Anchor Version (`programs/sol_diamond/`)
A minimal Anchor-based diamond router with:
- **diamond_state/** - Core state & access control
- **diamond_router/** - CPI-based dispatch logic  
- **diamond_cut/** - Dynamic facet management
- **error.rs** - Error definitions

### 2. Native Version (`native/`)
Pure Rust implementation proving framework independence:
- **router/** - Native diamond router (no Anchor)
- **facet/** - Example counter facet
- Complete documentation and build tools

---

## 📦 Structure

```
Solana_Diamond_Protocol_MVP/
├── programs/
│   └── sol_diamond/              # Minimal Anchor diamond router
│       ├── src/
│       │   ├── lib.rs            # Program entry point
│       │   ├── diamond_state/    # Core state & access control
│       │   ├── diamond_router/   # Dispatch logic
│       │   ├── diamond_cut/      # Module management
│       │   └── error.rs          # Error definitions
│       └── Cargo.toml
├── native/                        # Native Rust implementation
│   ├── router/                    # Native diamond router
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── diamond_state/
│   │   │   ├── diamond_router/
│   │   │   ├── diamond_cut/
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   ├── facet/                     # Example facet
│   │   └── src/lib.rs
│   ├── README.md
│   ├── EXAMPLE.md
│   └── build.sh
├── Anchor.toml                    # Anchor configuration
├── Cargo.toml                     # Workspace config
└── README.md                      # This file
```

---

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor --tag v0.31.1 anchor-cli
```

### Build Everything

```bash
# Clone and enter directory
cd Solana_Diamond_Protocol_MVP

# Build Anchor version
anchor build

# Build native version
cd native
./build.sh
```

### Test Locally

```bash
# Start local validator
solana-test-validator

# In another terminal
anchor test --skip-local-validator
```

---

## 🔍 Key Differences: Anchor vs Native

### What's the Same (Core Logic)

Both implementations share **identical dispatch logic**:

```rust
// 1. Extract selector from instruction data
let selector: [u8; 4] = ix_data[..4].try_into()?;

// 2. Lookup facet in registry
let facet_program = state.get_facet_by_selector(selector)?;

// 3. Validate and forward via CPI
invoke(&Instruction {
    program_id: facet_program,
    accounts: remaining_accounts,
    data: ix_data,
}, remaining_accounts)?;
```

### What's Different (Boilerplate)

| Aspect | Anchor | Native |
|--------|--------|--------|
| **Entry point** | `#[program]` macro | `entrypoint!()` + manual |
| **Account parsing** | `#[derive(Accounts)]` | Manual `next_account_info()` |
| **State** | `#[account]` | `BorshSerialize` + `BorshDeserialize` |
| **Build** | `anchor build` | `cargo build-sbf` |
| **Binary size** | ~150KB | ~80KB |

---

## 📚 Core Concepts

### 1. Diamond State

The central registry storing:
- **Owner** - Who can modify the diamond
- **Selectors** - Map of function selectors to facet programs
- **Modules** - Metadata about registered facets
- **Paused** - Emergency pause flag

### 2. Dispatch Pattern

```
Client → Diamond Router → Selector Lookup → CPI to Facet
```

The router:
1. Receives instruction with 4-byte selector
2. Looks up which facet handles that selector
3. Validates the provided facet matches registry
4. Forwards the instruction via Cross-Program Invocation (CPI)

### 3. Dynamic Facets

Add/remove facets at runtime:
- `add_facet()` - Register new functionality
- `remove_facet()` - Remove functionality (unless immutable)

---

## 🎓 Learning Path

### 1. Start with Anchor Version

```bash
# Build and explore
anchor build
code programs/sol_diamond/src/
```

**Key files to read:**
1. `lib.rs` - Program structure
2. `diamond_router/mod.rs` - Dispatch logic
3. `diamond_state/mod.rs` - State management

### 2. Compare with Native Version

```bash
cd native
code router/src/
```

**Notice:**
- Same folder structure (`diamond_state/`, `diamond_router/`, `diamond_cut/`)
- Same core logic
- Different entry point and account handling

### 3. Run Examples

```bash
# See native/EXAMPLE.md for detailed examples
cd native
cat EXAMPLE.md
```

---

## 🧪 Testing

### Anchor Tests

```bash
anchor test
```

### Native Tests

```bash
cd native
cargo test
```

### Integration Tests

See `tests/` directory for end-to-end examples.

---

## 🔧 Building for Production

### Anchor Version

```bash
anchor build --verifiable
anchor deploy --provider.cluster mainnet
```

### Native Version

```bash
cd native
cargo build-sbf --release
solana program deploy target/deploy/diamond_router_native.so
```

---

## 📖 Documentation

- **Anchor Implementation** - See `programs/sol_diamond/src/`
- **Native Implementation** - See `native/README.md` and `native/EXAMPLE.md`
- **Architecture Guide** - See `ARCHITECTURE.md` (in parent repo)

---

## 🎯 Use Cases

This MVP is perfect for:

- ✅ **Learning** the diamond pattern on Solana
- ✅ **Prototyping** modular protocols
- ✅ **Comparing** Anchor vs native approaches
- ✅ **Building** production diamond implementations

---

## 🤝 Contributing

This is a reference implementation. For contributions to the main protocol, see the parent repository.

---

## 📄 License

MIT

---

## 🙏 Acknowledgments

Based on EIP-2535 Diamond Standard, adapted for Solana's account-based model and CPI architecture.

---

**Built to demonstrate universal portability. Framework-agnostic by design.**
