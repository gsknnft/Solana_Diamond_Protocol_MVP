# Solana Diamond Protocol 💎

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust](https://img.shields.io/badge/rust-1.87.0-orange)
![Anchor](https://img.shields.io/badge/anchor-0.31.1-blue)
![License](https://img.shields.io/badge/license-MIT-green)

**A canonical, production-ready implementation of the EIP-2535 Diamond Standard for Solana**

Bringing Ethereum's Diamond Standard to Solana with native CPI-based facet routing, PDA state management, and independent program upgrades. This implementation enables modular, upgradeable smart contract architecture with hot-swappable facets while maintaining the core principles of EIP-2535.

> 📚 **New to Diamond?** Start with [README.diamond.md](./README.diamond.md) for a comprehensive MVP overview.
>
> 🎨 **Want to visualize the architecture?** See [FLOW_DIAGRAM.md](./FLOW_DIAGRAM.md) for detailed flow diagrams.
>
> 🤝 **Ready to contribute?** Check out [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## Branch Strategy
- **main** → Stable baseline, pinned to Anchor 0.31.1 for Solana 2.x compatibility.
- **diamond-erc2535** → Experimental branch with ERC‑2535 Diamond Standard implementation.
  - Includes selector collision detection, immutability flags, and human‑readable mappings.
  - Based on Anchor 0.32.1 with temporary compatibility fixes until 0.33.0.

## 🎯 Overview

The Solana Diamond Protocol provides a flexible framework for building complex, modular programs on Solana. It implements a proxy-style pattern where a central router dispatches calls to registered facet programs, allowing you to:

- ✅ **Add functionality** without redeploying the main contract
- ✅ **Upgrade modules** independently with zero downtime
- ✅ **Organize code** into logical, maintainable facets
- ✅ **Scale beyond** size limits with distributed logic
- ✅ **Share state** across multiple programs efficiently

## 🚀 Quick Start

```bash
# Clone and install
git clone https://github.com/gsknnft/Solana_Diamond_Protocol_MVP.git
cd Solana_Diamond_Protocol_MVP
npm install

# Build
cargo build

# Test
anchor test
```

See [QUICKSTART.md](./QUICKSTART.md) for detailed setup instructions.

## 📋 Features

### Core Diamond Router
- **Dynamic Dispatch**: Route instructions to registered facet programs via CPI
- **Selector Registry**: Map 4-byte function selectors to program addresses
- **Access Control**: Owner and admin-based permissions
- **Capacity Management**: Bounded collections prevent stack overflow
- **Safe Operations**: Validation ensures only registered facets are called

### Architecture Highlights
- **Modular Design**: Separate concerns into independent facets
- **Upgradeable**: Hot-swap facets without touching the router
- **Gas Efficient**: Optimized dispatch with O(n) lookup
- **Production Ready**: Thoroughly tested and documented

### Technical Specifications
- Maximum 10 admins per diamond
- Maximum 20 registered modules
- Maximum 50 function selectors
- Account size: 3,317 bytes
- Stack-safe: All operations within Solana limits

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         Diamond Router (Main)           │
│  ┌─────────────────────────────────┐   │
│  │     Dispatch Mechanism          │   │
│  │  1. Extract selector            │   │
│  │  2. Lookup facet address        │   │
│  │  3. Validate program ID         │   │
│  │  4. Forward via CPI             │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
   ┌────────┐  ┌────────┐  ┌────────┐
   │ Facet  │  │ Facet  │  │ Facet  │
   │   A    │  │   B    │  │   C    │
   └────────┘  └────────┘  └────────┘
```

For detailed architecture documentation, see [ARCHITECTURE.md](./ARCHITECTURE.md).

## 📦 Project Structure

```
Solana_Diamond_Protocol_MVP/
├── programs/
│   ├── sol_diamond/           # Main diamond router program
│   │   ├── src/
│   │   │   ├── lib.rs         # Program entry point
│   │   │   ├── diamond_state/ # Core state & access control
│   │   │   ├── diamond_router/# Dispatch logic
│   │   │   ├── diamond_cut/   # Module management
│   │   │   └── error.rs       # Error definitions
│   │   └── Cargo.toml
│   ├── rewards_facet/         # Example facet: IMG rewards
│   └── lp_rewards_facet/      # Example facet: LP staking
├── tests/
│   └── sol_diamond.ts         # Integration tests
├── ARCHITECTURE.md            # Detailed design docs
├── QUICKSTART.md              # Setup guide
└── README.md                  # This file
```

## 🔧 Usage Example

### Initialize the Diamond

```typescript
import * as anchor from "@coral-xyz/anchor";

const program = anchor.workspace.SolDiamond;
const owner = anchor.web3.Keypair.generate();

// Initialize diamond state
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

### Register a Facet

```typescript
const facetProgramId = new PublicKey("YourFacetProgramID");
const selector = [0x01, 0x02, 0x03, 0x04]; // 4-byte function selector

await program.methods
  .addModule(facetProgramId, selector)
  .accounts({
    diamondState: diamondStatePDA,
    authority: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

### Dispatch to a Facet

```typescript
const ixData = Buffer.concat([
  Buffer.from(selector),
  Buffer.from(encodedArgs)
]);

await program.methods
  .dispatch(Array.from(ixData))
  .accounts({
    routerConfig: diamondStatePDA,
    module: facetProgramId,
  })
  .remainingAccounts([...facetAccounts])
  .rpc();
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Unit tests
cargo test

# Integration tests with Anchor
anchor test

# Specific test file
anchor test tests/sol_diamond.ts

# Verbose output with logs
anchor test -- --nocapture
```

Tests cover:
- Diamond initialization
- Admin management
- Module registration
- Selector mapping
- Dispatch routing
- Access control
- Capacity limits

## 📦 Building & IDL Generation

### Build Programs

```bash
# Build all programs in workspace
anchor build

# Build specific program
cargo build-sbf --manifest-path programs/sol_diamond/Cargo.toml

# Build for production (optimized)
cargo build-sbf --release
```

### Generate IDLs

Anchor automatically generates Interface Definition Language (IDL) files for client integration:

```bash
# IDLs are generated during anchor build
anchor build

# IDL files location
ls -la target/idl/
# - sol_diamond.json
# - img_rewards_facet.json
# - lp_rewards_facet.json
```

### Using IDLs in Client Code

```typescript
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { IDL as DiamondIDL } from "./target/idl/sol_diamond";

const provider = AnchorProvider.env();
const program = new Program(DiamondIDL, provider);

// Now you have typed access to all instructions
await program.methods.initializeDiamond(owner, bump).accounts({...}).rpc();
```

**IDL Configuration**: See `Anchor.toml` for IDL output settings:
```toml
[idl]
out = "target/idl"
```

## 🛣️ Roadmap

### Phase 1: Core Protocol ✅ COMPLETE
- [x] Diamond router implementation
- [x] Dispatch mechanism
- [x] Access control
- [x] Stack optimization
- [x] Test suite
- [x] Documentation

### Phase 2: Facet Ecosystem (In Progress)
- [ ] FacetRegistry PDA for scaling
- [ ] DiamondCut facet for complex upgrades
- [ ] CrossChainBridge facet (SigilNet integration)
- [ ] Enhanced selector management

### Phase 3: Advanced Features
- [ ] LP rewards facet
- [ ] Governance facet (DAO voting)
- [ ] Snapshot facet (holder tracking)
- [ ] Automated epoch management

### Phase 4: Production Deployment
- [ ] Security audit
- [ ] Mainnet deployment
- [ ] Community launch
- [ ] SDK and tooling

## 🔒 Security

### Implemented Safeguards
- **Bounded Collections**: Prevent DOS attacks with max limits (10 admins, 20 modules, 50 selectors)
- **Selector Validation**: Ensures safe dispatch by validating program IDs
- **Access Control**: Owner-only critical operations with admin tier
- **Capacity Limits**: Prevent resource exhaustion and stack overflow
- **Stack Safety**: All operations within Solana's 4KB stack constraints
- **Namespace Partitioning**: Prevents selector collisions (EIP-2535 §5 compliant)
- **Emergency Pause**: Owner can freeze all dispatches with reason message
- **Immutability Flags**: Mark critical functions as non-replaceable

### Audit Status
This is beta software under active development. A formal security audit is planned before mainnet deployment. Use at your own risk in production environments.

### Responsible Disclosure

**Found a security vulnerability?** Please report it responsibly:

**📧 Email**: gsknnft@gmail.com

**Please include:**
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (optional)

We aim to respond within 48 hours. Security researchers will be credited in our security advisories.

**DO NOT** open public GitHub issues for security vulnerabilities.

## 🤝 Contributing

We welcome contributions from the community! Whether you're fixing bugs, building facets, or improving documentation, your help is appreciated.

**👉 See [CONTRIBUTING.md](./CONTRIBUTING.md) for comprehensive guidelines including:**
- Development setup and workflow
- Coding standards and best practices
- Testing requirements
- Pull request process
- Facet development guide
- Security guidelines

### Quick Contribution Steps

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Ensure all tests pass (`anchor test`)
5. Submit a pull request

### Areas We Need Help With

- 🔧 **Facet Development**: Build new facet programs (DEX, lending, governance)
- 📝 **Documentation**: Improve guides, add examples, fix typos
- 🧪 **Testing**: Add test cases, improve coverage, fuzzing
- 🔒 **Security**: Review code, report vulnerabilities, suggest improvements
- 🛠️ **Tooling**: CLI tools, SDK enhancements, deployment scripts

## 📚 Documentation

### Essential Reading
- **[README.diamond.md](./README.diamond.md)** - **START HERE**: Comprehensive MVP overview with architecture, flows, and examples
- **[FLOW_DIAGRAM.md](./FLOW_DIAGRAM.md)** - Visual flow diagrams for all key processes
- **[CONTRIBUTING.md](./CONTRIBUTING.md)** - Complete contributor guide
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Detailed technical design and implementation
- **[QUICKSTART.md](./QUICKSTART.md)** - Setup guide and basic usage

### Additional Resources
- **[SECURITY_REVIEW.md](./SECURITY_REVIEW.md)** - Security analysis and considerations
- **[WHITEPAPER_OUTLINE.md](./WHITEPAPER_OUTLINE.md)** - Theoretical foundation
- **EIP-2535 Standard**: [eips.ethereum.org/EIPS/eip-2535](https://eips.ethereum.org/EIPS/eip-2535)

## 🌟 Use Cases

### Token Ecosystems
- Modular token with swappable transfer logic
- Tax and fee mechanisms as facets
- Rewards distribution modules

### DeFi Protocols
- LP staking and rewards
- Multi-strategy yield farming
- Cross-chain bridge integration

### DAO Infrastructure
- Upgradeable governance
- Modular proposal types
- Treasury management facets

### Gaming
- Upgradeable game logic
- Item and inventory systems
- Quest and achievement modules

## 🔗 Links

- **GitHub**: [gsknnft/Solana_Diamond_Protocol_MVP](https://github.com/gsknnft/Solana_Diamond_Protocol_MVP)
- **SigilNet**: [Cross-chain integration](https://github.com/gsknnft/SigilNet)
- **EIP-2535**: [Diamond Standard](https://eips.ethereum.org/EIPS/eip-2535)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by Nick Mudge's EIP-2535 Diamond Standard
- Built with the Anchor Framework
- Special thanks to the Solana developer community

## 💬 Community & Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/gsknnft/Solana_Diamond_Protocol_MVP/issues)
- **GitHub Discussions**: [Ask questions and share ideas](https://github.com/gsknnft/Solana_Diamond_Protocol_MVP/discussions)
- **Security**: security@sigilnet.io (for vulnerability reports only)

---

## 🎯 Why This Implementation Matters

This repository represents a **canonical implementation** of the Diamond Standard for Solana, demonstrating:

- 🧭 **Clear Intent**: Production-ready architecture following EIP-2535 principles
- 🏗️ **Sovereign Design**: Native Solana patterns (CPI, PDAs) instead of Ethereum workarounds
- 📐 **Clean Architecture**: Minimal, focused, ready for contributors
- 🔐 **Security-First**: Comprehensive safeguards and validation
- 📚 **Well-Documented**: Complete guides for understanding and contributing
- 🧬 **Proven Authorship**: Deep understanding of both Diamond Standard and Solana

Built for the Solana ecosystem by developers who understand modular smart contract architecture.

---

**Built with ❤️ for the Solana ecosystem**

*Canonical Diamond Standard implementation demonstrating production-ready patterns, sovereign design, and clear upgrade strategies.*

