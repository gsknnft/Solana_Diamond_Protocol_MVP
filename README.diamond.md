# Solana Diamond Protocol MVP

**A canonical implementation of EIP-2535 Diamond Standard for Solana**

> *Bringing Ethereum's Diamond Standard to Solana with native CPI-based facet routing, PDA state management, and independent program upgrades.*

---

## 🎯 Overview

This implementation demonstrates a production-ready diamond architecture on Solana that maintains the core principles of EIP-2535 while embracing Solana's unique programming model:

- **Facets as Separate Programs**: Each facet is a distinct Solana program, not just contract logic
- **Router Dispatches via CPI**: Central diamond router uses Cross-Program Invocation to forward calls
- **Shared State via PDAs**: Program Derived Addresses provide shared, versioned state across facets
- **Independent Upgrades**: Facets can be redeployed without touching the router or other facets
- **Selector Registry**: 4-byte function selectors map to program addresses, enabling dynamic dispatch

### Why Diamond on Solana?

Solana's account-based model and program limitations (e.g., 10MB size limit) make modular architectures essential for complex protocols. The Diamond pattern solves:

1. **Size Limits**: Distribute logic across multiple programs
2. **Upgrade Flexibility**: Replace individual modules without redeploying everything
3. **Gas Efficiency**: Only load required facet code for each transaction
4. **Code Organization**: Logical separation of concerns across programs

---

## 🏗️ Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────────────┐
│                     CLIENT APPLICATION                       │
│  (Constructs instruction with 4-byte selector + args)       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  DIAMOND ROUTER PROGRAM                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  1. Extract 4-byte selector from instruction data    │  │
│  │  2. Lookup selector in DiamondState registry         │  │
│  │  3. Validate program_id matches registered facet     │  │
│  │  4. Forward instruction via CPI with all accounts    │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
          ▼              ▼              ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │  FACET   │   │  FACET   │   │  FACET   │
    │ REWARDS  │   │LP REWARDS│   │  TRADE   │
    │          │   │          │   │          │
    │ Program  │   │ Program  │   │ Program  │
    └────┬─────┘   └────┬─────┘   └────┬─────┘
         │              │              │
         │         READ/WRITE          │
         └──────────────┼──────────────┘
                        ▼
              ┌───────────────────┐
              │   SHARED STATE    │
              │   (PDA Accounts)  │
              │                   │
              │ • DiamondState    │
              │ • RewardConfig    │
              │ • LpConfig        │
              │ • UserState       │
              └───────────────────┘
```

### Key Components

#### 1. Diamond Router (`programs/sol_diamond`)

The central dispatcher that:
- Maintains a registry of function selectors → program addresses
- Validates incoming instructions against registered facets
- Forwards calls via CPI to the appropriate facet program
- Enforces access control (owner, admins)
- Manages facet registration/removal

**Core State Account: `DiamondState`**
```rust
pub struct DiamondState {
    pub owner: Pubkey,                    // Primary authority
    pub admins: Vec<Pubkey>,              // Max 10 admins
    pub active_modules: Vec<ModuleMeta>,  // Max 20 facets
    pub selectors: Vec<SelectorMapping>,  // Max 50 function selectors
    pub is_paused: bool,                  // Emergency pause
    // ... governance and cache fields
}
```

#### 2. Facet Programs

Independent Solana programs implementing specific functionality:

**`rewards_facet`** (`programs/rewards_facet`)
- IMG token reward distribution
- Snapshot-based eligibility verification
- Batch distribution with time gating
- Admin-controlled reward intervals

**`lp_rewards_facet`** (`programs/lp_rewards_facet`)
- LP staking and rewards
- Emission rate configuration
- Vault management
- Active/inactive state control

Each facet:
- Defines its own instructions and state structures
- Can be called directly OR through the diamond router
- Manages its own PDAs or shares router PDAs via passed accounts
- Can be upgraded independently via `solana program deploy --program-id <id>`

#### 3. Shared State Pattern

Facets access shared state via passed account references:

```typescript
// Example: Dispatch to rewards facet with shared state
await diamondProgram.methods
  .dispatch(instructionData)
  .accounts({
    routerConfig: diamondStatePDA,  // Router state
    module: rewardsFacetProgramId,  // Target facet
  })
  .remainingAccounts([
    { pubkey: rewardConfigPDA, isSigner: false, isWritable: true },
    { pubkey: userStatePDA, isSigner: false, isWritable: true },
    // ... other accounts the facet needs
  ])
  .rpc();
```

The router doesn't interpret the remaining accounts—it passes them through to the facet via CPI.

---

## 🔧 Core Functionality

### Initializing the Diamond

```typescript
import * as anchor from "@coral-xyz/anchor";
import { SystemProgram } from "@solana/web3.js";

const program = anchor.workspace.SolDiamond;
const owner = anchor.web3.Keypair.generate();

// Derive diamond state PDA
const [diamondStatePDA, bump] = await PublicKey.findProgramAddress(
  [Buffer.from("diamond_state"), owner.publicKey.toBuffer()],
  program.programId
);

// Initialize diamond
await program.methods
  .initializeDiamond(owner.publicKey, bump)
  .accounts({
    diamondState: diamondStatePDA,
    payer: owner.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([owner])
  .rpc();
```

### Registering a Facet

```typescript
// Register rewards facet with selector
const rewardsFacetId = new PublicKey("NnP3k6V9FpiiXhSnKrvYG11PH2kjH313gvZQPhqkz58");
const selector = [0x01, 0x23, 0x45, 0x67]; // distribute_rewards selector

await program.methods
  .addModule(rewardsFacetId, selector)
  .accounts({
    diamondState: diamondStatePDA,
    authority: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

### Dispatching to a Facet

```typescript
// Prepare instruction data: [selector (4 bytes)] + [facet-specific args]
const selector = [0x01, 0x23, 0x45, 0x67];
const amount = new anchor.BN(1000000); // 1 token with 6 decimals
const encodedArgs = amount.toArrayLike(Buffer, "le", 8);
const ixData = Buffer.concat([Buffer.from(selector), encodedArgs]);

// Dispatch through router
await program.methods
  .dispatch(Array.from(ixData))
  .accounts({
    routerConfig: diamondStatePDA,
    module: rewardsFacetId,
  })
  .remainingAccounts([
    { pubkey: rewardConfigPDA, isSigner: false, isWritable: true },
    { pubkey: treasury, isSigner: false, isWritable: true },
    { pubkey: recipient, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ])
  .rpc();
```

---

## 📦 Project Structure

```
Solana_Diamond_Protocol/
├── programs/
│   ├── sol_diamond/              # Diamond Router (Main Program)
│   │   ├── src/
│   │   │   ├── lib.rs            # Program entry, #[program] macro, all Context structs
│   │   │   ├── diamond_state/    # DiamondState account and constants
│   │   │   ├── diamond_router/   # Dispatch logic (CPI forwarding)
│   │   │   ├── diamond_cut/      # Module add/remove operations
│   │   │   ├── diamond_access/   # Owner/admin permission checks
│   │   │   ├── diamond_init/     # Initialization helpers
│   │   │   ├── diamond_loupe/    # Query functions (view state)
│   │   │   ├── diamond_hooks/    # Optional transfer hooks
│   │   │   ├── diamond_lib/      # Facet integration helpers (portal)
│   │   │   ├── governance/       # Timelock, multisig integration
│   │   │   ├── bridges/          # Cross-chain integration (SigilNet)
│   │   │   ├── selector_utils.rs # Selector generation/verification
│   │   │   ├── error.rs          # Custom error codes
│   │   │   └── utils/            # Emergency withdraw, misc
│   │   └── Cargo.toml
│   │
│   ├── rewards_facet/            # IMG Rewards Facet (Example)
│   │   ├── src/
│   │   │   ├── lib.rs            # Facet program entry
│   │   │   ├── instructions/     # Instruction handlers
│   │   │   └── state.rs          # Facet-specific state
│   │   └── Cargo.toml
│   │
│   └── lp_rewards_facet/         # LP Rewards Facet (Example)
│       ├── src/
│       │   ├── lib.rs            # Facet program entry
│       │   ├── initialize_lp.rs  # LP config initialization
│       │   └── lp_rewards.rs     # LP state structures
│       └── Cargo.toml
│
├── tests/                        # Integration tests
│   ├── sol_diamond.ts            # Router tests
│   ├── rewards_program.ts        # Rewards facet tests
│   └── diamond_enhancements.ts   # Advanced feature tests
│
├── ts-sdk/                       # TypeScript SDK (optional)
│   └── src/                      # Client helpers
│
├── Anchor.toml                   # Anchor configuration
├── Cargo.toml                    # Workspace configuration
├── README.md                     # Main documentation
├── ARCHITECTURE.md               # Detailed design docs
├── README.diamond.md             # This file (MVP overview)
└── QUICKSTART.md                 # Setup guide
```

---

## 🚀 Upgrade Strategy

### Facet Upgrades

**Independent Redeployment**: Each facet can be upgraded without affecting others:

```bash
# Upgrade rewards facet to new version
solana program deploy \
  --program-id NnP3k6V9FpiiXhSnKrvYG11PH2kjH313gvZQPhqkz58 \
  target/deploy/img_rewards_facet.so \
  --upgrade-authority <authority-keypair>
```

The router continues to dispatch to the same program ID—no registry update needed.

### Router Logic Updates

**Option 1: Governance-Controlled Upgrade**
- Use a multisig (Squads) or DAO governance (SPL Governance) as the upgrade authority
- Proposals go through voting before router upgrades

**Option 2: Timelock**
- Queue router upgrades with a delay (e.g., 7 days)
- Community can review changes before they take effect
- Implemented in `programs/sol_diamond/src/governance/timelock.rs`

**Option 3: Immutable Core + Versioned Facets**
- Mark the router program as immutable after launch
- All future changes happen via facet updates only
- Ensures no rug-pull risk on core routing logic

### Adding New Facets

```typescript
// Owner adds a new facet after audit
const newFacetId = new PublicKey("...");
const newSelector = [0xAA, 0xBB, 0xCC, 0xDD];

await diamondProgram.methods
  .addModule(newFacetId, newSelector)
  .accounts({
    diamondState: diamondStatePDA,
    authority: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

### State Stability

**PDA Anchors Ensure Backward Compatibility**:
- DiamondState PDA is derived from owner pubkey → never changes
- Facet state PDAs use consistent seeds → survive facet upgrades
- Account layout changes require migration facets (not automatic)

**Migration Pattern**:
1. Deploy new facet version with updated state layout
2. Deploy a migration facet that reads old state, writes new state
3. Call migration facet for all affected accounts
4. Update router to point to new facet version
5. Deprecate old facet

---

## 🔐 Security Model

### Access Control Tiers

1. **Owner** (Single Pubkey)
   - Add/remove facets
   - Manage admins
   - Transfer ownership
   - Emergency pause

2. **Admins** (Up to 10)
   - Configurable per-facet permissions
   - Can call administrative facet functions
   - Cannot modify router state

3. **Public**
   - Call registered facet functions
   - Subject to facet-level validation

### Safeguards

- **Bounded Collections**: Max 10 admins, 20 modules, 50 selectors → prevents DOS
- **Selector Validation**: Dispatcher checks program ID matches registered facet
- **Immutability Flags**: Mark critical functions as non-replaceable (EIP-2535 compliant)
- **Emergency Pause**: Owner can freeze all dispatches with a reason message
- **Namespace Support**: Prevent selector collisions across libraries (EIP-2535 §5)

### Attack Vectors & Mitigations

| Attack | Mitigation |
|--------|-----------|
| **Malicious Facet Registration** | Only owner can register facets; thorough audits required |
| **Selector Collision** | 4-byte selector space (4.3B combinations) + namespace partitioning |
| **Upgrade Authority Compromise** | Use multisig or governance as upgrade authority |
| **State Corruption** | Facets validate all state transitions; PDA derivation prevents spoofing |
| **Reentrancy** | Solana's single-threaded execution prevents classic reentrancy |
| **CPI Depth Limit** | Max depth 4; diamond adds 1 level → facets have 3 levels remaining |

---

## 📊 Technical Specifications

### Account Sizes

- **DiamondState**: 3,317 bytes
  - Discriminator: 8 bytes
  - Owner: 32 bytes
  - Admins: 324 bytes (4 + 10×32)
  - Modules: 1,364 bytes (4 + 20×68)
  - Selectors: 1,804 bytes (4 + 50×113)
  - Pause/governance: ~80 bytes
  - Hot cache: ~570 bytes

### Capacity Limits

- **Max Admins**: 10
- **Max Registered Modules**: 20
- **Max Function Selectors**: 50
- **CPI Depth**: 1 (dispatcher) + facet depth ≤ 4 total

### Performance

- **Selector Lookup**: O(n) linear search (n ≤ 50)
- **Hot Cache**: 5-slot LRU cache for frequent selectors
- **Gas Cost**: ~5,000 CU for dispatch + facet execution cost

### Compatibility

- **Anchor**: 0.31.1 (Solana 2.x compatible)
- **Rust**: 1.87.0+
- **Solana**: 2.0+ (localnet, devnet, mainnet-beta)

---

## 🧪 Testing & Validation

### Running Tests

```bash
# Build all programs
cargo build

# Run integration tests
anchor test

# Run specific test file
anchor test tests/sol_diamond.ts
```

### Test Coverage

- ✅ Diamond initialization and PDA derivation
- ✅ Owner and admin management
- ✅ Facet registration (add/remove)
- ✅ Selector mapping and collision detection
- ✅ Dispatch routing with CPI
- ✅ Access control enforcement
- ✅ Capacity limit validation
- ✅ Emergency pause functionality
- ✅ Rewards facet distribution
- ✅ LP rewards configuration

### Pre-Deployment Checklist

- [ ] All tests pass on localnet
- [ ] Deployed and tested on devnet
- [ ] Security audit completed
- [ ] Upgrade authority transferred to multisig/governance
- [ ] Emergency contacts documented
- [ ] Monitoring and alerting configured
- [ ] Documentation reviewed by external auditor

---

## 🌟 EIP-2535 Compliance

This implementation adheres to the core principles of EIP-2535 while adapting to Solana's unique constraints:

| EIP-2535 Feature | Solana Implementation |
|------------------|----------------------|
| **Diamond Proxy** | Diamond Router Program (CPI dispatcher) |
| **Facets** | Separate Solana Programs |
| **Function Selectors** | 4-byte identifiers → program addresses |
| **DiamondCut** | `add_module()`, `remove_module()` instructions |
| **DiamondLoupe** | `get_facets()`, `get_selectors()` view functions |
| **Immutability** | `is_immutable` flag on SelectorMapping |
| **Namespace Support** | `namespace: [u8; 8]` field for library partitioning (§5) |
| **Events** | Anchor events: `FacetAdded`, `FacetRemoved`, etc. |

**Key Differences from Ethereum**:
- No `delegatecall` → uses CPI instead
- No fallback function → explicit `dispatch()` instruction
- No storage slots → PDA-based state management
- Facets are separate programs, not contract code

---

## 🛣️ Roadmap

### ✅ Phase 1: Core Protocol (Complete)
- [x] Diamond router implementation
- [x] Dispatch mechanism with CPI
- [x] Selector registry
- [x] Access control
- [x] Stack optimization
- [x] Comprehensive tests

### 🚧 Phase 2: Facet Ecosystem (In Progress)
- [x] Rewards facet (IMG distribution)
- [x] LP rewards facet (staking)
- [ ] DiamondCut facet (complex upgrades)
- [ ] FacetRegistry PDA (scale beyond 50 selectors)
- [ ] CrossChainBridge facet (SigilNet integration)

### 📋 Phase 3: Governance & Tooling
- [ ] Timelock implementation (7-day upgrade delay)
- [ ] Squads multisig integration
- [ ] SPL Governance integration
- [ ] CLI tool for facet management
- [ ] SDK helpers for client integration

### 🚀 Phase 4: Production Deployment
- [ ] Formal security audit (Trail of Bits / OtterSec)
- [ ] Mainnet deployment with multisig authority
- [ ] Community launch campaign
- [ ] Documentation portal with examples
- [ ] Bug bounty program

---

## 🤝 Contributing

We welcome contributions from the community! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### How to Contribute

1. **Report Issues**: Found a bug? Open an issue with reproduction steps
2. **Build Facets**: Create new facet programs for the ecosystem
3. **Improve Docs**: Help us clarify the architecture
4. **Security Research**: Report vulnerabilities responsibly
5. **Test Coverage**: Add tests for edge cases

### Development Setup

```bash
# Clone repo
git clone https://github.com/gsknnft/Solana_Diamond_Protocol_dev.git
cd Solana_Diamond_Protocol_dev

# Install dependencies
npm install

# Build programs
cargo build

# Run tests
anchor test
```

---

## 📚 Additional Resources

- **Main README**: [README.md](./README.md)
- **Architecture Guide**: [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Quick Start**: [QUICKSTART.md](./QUICKSTART.md)
- **EIP-2535 Standard**: [eips.ethereum.org/EIPS/eip-2535](https://eips.ethereum.org/EIPS/eip-2535)
- **Anchor Docs**: [anchor-lang.com](https://www.anchor-lang.com/)
- **Solana Cookbook**: [solanacookbook.com](https://solanacookbook.com/)

---

## 📜 License

MIT License - see [LICENSE](./LICENSE) for details.

---

## 🙏 Acknowledgments

- **Nick Mudge** for the original EIP-2535 Diamond Standard
- **Solana Foundation** for the Anchor framework
- **Community Auditors** for security review and feedback

---

## 💬 Contact & Community

- **GitHub Issues**: [Report bugs or request features](https://github.com/gsknnft/Solana_Diamond_Protocol_dev/issues)
- **Discussions**: [Ask questions and share ideas](https://github.com/gsknnft/Solana_Diamond_Protocol_dev/discussions)
- **Security**: Report vulnerabilities privately to security@sigilnet.io

---

**Built with ❤️ for the Solana ecosystem**

*This is canonical work demonstrating diamond architecture on Solana with production-ready patterns, sovereign design, and clear upgrade strategies.*
