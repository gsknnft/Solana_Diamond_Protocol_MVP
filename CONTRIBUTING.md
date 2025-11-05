# Contributing to Solana Diamond Protocol

Thank you for your interest in contributing to the Solana Diamond Protocol! This document provides guidelines and best practices for contributing to this project.

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Submitting Changes](#submitting-changes)
- [Building Facets](#building-facets)
- [Security Guidelines](#security-guidelines)
- [Documentation](#documentation)

---

## 🤝 Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please:

- Be respectful and considerate in all interactions
- Focus on constructive feedback
- Help newcomers get started
- Respect differing viewpoints and experiences

---

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.87.0 or higher
- **Solana CLI**: 2.0+
- **Anchor**: 0.31.1
- **Node.js**: 18+ and npm/pnpm
- **Git**: For version control

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/gsknnft/Solana_Diamond_Protocol_dev.git
cd Solana_Diamond_Protocol_dev

# Install dependencies
npm install

# Build programs
cargo build

# Run tests to verify setup
anchor test
```

### Understanding the Architecture

Before contributing, please read:
1. [README.md](./README.md) - High-level overview
2. [README.diamond.md](./README.diamond.md) - MVP architecture
3. [ARCHITECTURE.md](./ARCHITECTURE.md) - Detailed design
4. [QUICKSTART.md](./QUICKSTART.md) - Setup guide

---

## 🔄 Development Process

### 1. Fork and Branch

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/Solana_Diamond_Protocol_dev.git
cd Solana_Diamond_Protocol_dev

# Create a feature branch
git checkout -b feature/your-feature-name
```

### 2. Make Changes

- Keep changes focused and atomic
- Write clear, descriptive commit messages
- Test thoroughly before committing
- Update documentation as needed

### 3. Test Locally

```bash
# Build all programs
cargo build

# Run full test suite
anchor test

# Run specific test
anchor test tests/sol_diamond.ts

# Check for compilation warnings
cargo clippy

# Format code
cargo fmt
```

### 4. Submit Pull Request

- Push your branch to your fork
- Open a PR against the `main` branch
- Fill out the PR template completely
- Link any related issues

---

## 📝 Coding Standards

### Rust Style

We follow standard Rust conventions:

```rust
// Good: Clear function names, proper error handling
pub fn add_module(
    ctx: Context<AddModule>,
    module_address: Pubkey,
    selector: [u8; 4],
) -> Result<()> {
    require!(
        ctx.accounts.diamond_state.selectors.len() < DiamondState::MAX_SELECTORS,
        DiamondError::SelectorCapacityExceeded
    );
    
    // Implementation...
    Ok(())
}

// Bad: Unclear names, unwrap() instead of proper errors
pub fn add(ctx: Context<Add>, addr: Pubkey, sel: [u8; 4]) -> Result<()> {
    let state = ctx.accounts.state;
    state.sels.push(sel).unwrap();  // ❌ Don't use unwrap()
    Ok(())
}
```

### Key Guidelines

- **No `unwrap()`**: Use proper error handling with custom errors
- **Bounded Collections**: Always enforce maximum capacities
- **Clear Naming**: Functions and variables should be self-documenting
- **Comments**: Add comments for complex logic, avoid obvious comments
- **Error Messages**: Provide helpful context in error strings
- **Space Calculations**: Document account size calculations

### TypeScript/JavaScript Style

```typescript
// Good: Typed, clear structure
interface DispatchArgs {
  selector: number[];
  args: Buffer;
}

async function dispatchToFacet(
  program: Program,
  args: DispatchArgs
): Promise<string> {
  const ixData = Buffer.concat([
    Buffer.from(args.selector),
    args.args
  ]);
  
  return await program.methods
    .dispatch(Array.from(ixData))
    .accounts({ /* ... */ })
    .rpc();
}

// Bad: Untyped, unclear
async function dispatch(prog, sel, arg) {
  return await prog.methods
    .dispatch([...sel, ...arg])
    .rpc();
}
```

---

## 🧪 Testing Requirements

### Test Coverage

All new features must include tests:

```typescript
describe("New Feature", () => {
  it("should succeed with valid inputs", async () => {
    // Arrange
    const selector = [0x01, 0x02, 0x03, 0x04];
    
    // Act
    const tx = await program.methods
      .addModule(facetProgramId, selector)
      .accounts({
        diamondState: diamondStatePDA,
        authority: owner.publicKey,
      })
      .signers([owner])
      .rpc();
    
    // Assert
    const state = await program.account.diamondState.fetch(diamondStatePDA);
    expect(state.selectors.length).to.equal(1);
  });
  
  it("should fail with unauthorized signer", async () => {
    // Test error case
    await expect(
      program.methods
        .addModule(facetProgramId, selector)
        .accounts({
          diamondState: diamondStatePDA,
          authority: unauthorized.publicKey,
        })
        .signers([unauthorized])
        .rpc()
    ).to.be.rejectedWith(/UnauthorizedAccess/);
  });
});
```

### Test Categories

1. **Unit Tests**: Test individual functions in isolation
2. **Integration Tests**: Test program interactions
3. **Edge Cases**: Test boundary conditions and limits
4. **Error Cases**: Verify proper error handling

### Running Tests

```bash
# All tests
anchor test

# Verbose output
anchor test -- --nocapture

# Specific test file
anchor test tests/sol_diamond.ts

# With local validator logs
anchor test --skip-local-validator
```

---

## 📤 Submitting Changes

### Pull Request Process

1. **Update Documentation**: If you change APIs or behavior
2. **Add Tests**: For all new functionality
3. **Run Full Suite**: Ensure all tests pass
4. **Format Code**: Run `cargo fmt` and `prettier`
5. **Write Good Commits**: Follow conventional commits

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples**:
```
feat(router): add namespace support for selector partitioning

Implements EIP-2535 §5 library partitioning with 8-byte namespace
hashes to prevent selector collisions across different facet libraries.

Closes #123
```

```
fix(dispatch): validate program_id before CPI invocation

Adds validation to ensure the passed module account matches the
registered program address in the selector mapping, preventing
unauthorized facet execution.

Fixes #456
```

### PR Template

Your PR description should include:

```markdown
## Description
Brief summary of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] All tests pass locally
- [ ] Added tests for new functionality
- [ ] Tested on devnet (if applicable)

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings introduced
```

---

## 🏗️ Building Facets

### Facet Development Guidelines

When creating a new facet program:

#### 1. Project Structure

```
programs/your_facet/
├── Cargo.toml
└── src/
    ├── lib.rs           # Program entry point
    ├── instructions/    # Instruction handlers
    │   ├── mod.rs
    │   ├── initialize.rs
    │   └── process.rs
    └── state.rs         # Facet-specific state
```

#### 2. State Design

```rust
// Use PDAs for state management
#[account]
pub struct YourFacetState {
    pub authority: Pubkey,
    pub config_value: u64,
    pub bump: u8,
}

impl YourFacetState {
    pub const SPACE: usize = 8 + 32 + 8 + 1;
}
```

#### 3. Selector Generation

Use consistent selector generation:

```rust
use sol_diamond::selector_utils::generate_selector;

// In your instruction
pub fn your_function(ctx: Context<YourContext>) -> Result<()> {
    // Your logic here
    Ok(())
}

// Generate selector (in tests or docs):
// let selector = generate_selector("your_function");
// => [0xAA, 0xBB, 0xCC, 0xDD]
```

#### 4. Integration with Router

```typescript
// Register your facet with the diamond
const yourFacetId = new PublicKey("YourFacetProgramId");
const selector = [0xAA, 0xBB, 0xCC, 0xDD];

await diamondProgram.methods
  .addModule(yourFacetId, selector)
  .accounts({
    diamondState: diamondStatePDA,
    authority: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

#### 5. Testing Your Facet

```typescript
describe("Your Facet", () => {
  it("should initialize correctly", async () => {
    await yourFacetProgram.methods
      .initialize(config)
      .accounts({ /* ... */ })
      .rpc();
  });
  
  it("should work through diamond router", async () => {
    const ixData = Buffer.concat([
      Buffer.from(selector),
      encodeArgs(yourArgs)
    ]);
    
    await diamondProgram.methods
      .dispatch(Array.from(ixData))
      .accounts({
        routerConfig: diamondStatePDA,
        module: yourFacetId,
      })
      .remainingAccounts([/* facet accounts */])
      .rpc();
  });
});
```

### Facet Best Practices

- ✅ Design facets to be stateless or use their own PDAs
- ✅ Document all required accounts and constraints
- ✅ Implement proper error handling
- ✅ Test independently before router integration
- ✅ Keep facets focused on a single responsibility
- ✅ Use versioning in state structures for future upgrades
- ❌ Don't assume router state layout
- ❌ Don't hardcode program IDs (use parameters)
- ❌ Don't exceed CPI depth limit (remember router uses 1 level)

---

## 🔒 Security Guidelines

### Security Review Process

All security-sensitive changes require:

1. **Self-Review**: Check for common vulnerabilities
2. **Peer Review**: At least one other developer must review
3. **Test Coverage**: Include security test cases
4. **Documentation**: Document security assumptions

### Common Security Checks

#### Access Control
```rust
// ✅ Good: Explicit authority checks
require!(
    ctx.accounts.diamond_state.owner == ctx.accounts.authority.key(),
    DiamondError::UnauthorizedAccess
);

// ❌ Bad: Missing checks
pub fn critical_operation(ctx: Context<Op>) -> Result<()> {
    // Anyone can call this!
    ctx.accounts.diamond_state.owner = ctx.accounts.new_owner.key();
    Ok(())
}
```

#### Integer Overflow
```rust
// ✅ Good: Checked arithmetic
let new_balance = balance
    .checked_add(amount)
    .ok_or(DiamondError::Overflow)?;

// ❌ Bad: Unchecked arithmetic
let new_balance = balance + amount;  // Can overflow!
```

#### Account Validation
```rust
// ✅ Good: Validate program ownership
require!(
    ctx.accounts.token_account.owner == spl_token::ID,
    DiamondError::InvalidTokenAccount
);

// ❌ Bad: Trusting unchecked accounts
/// CHECK: This is unsafe!
pub token_account: AccountInfo<'info>,
```

### Reporting Security Issues

**DO NOT** open public issues for security vulnerabilities.

Instead, email: **security@sigilnet.io** with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We aim to respond within 48 hours and will credit reporters in our security advisories.

---

## 📚 Documentation

### Documentation Requirements

All code contributions should include:

1. **Inline Comments**: For complex logic
2. **Function Documentation**: Rust doc comments
3. **README Updates**: If changing public APIs
4. **Architecture Docs**: For significant design changes

### Rust Documentation

```rust
/// Adds a new facet to the diamond registry.
///
/// This function registers a new facet program with a 4-byte selector,
/// enabling the diamond router to dispatch calls to it.
///
/// # Arguments
/// * `module_address` - The program ID of the facet to register
/// * `selector` - 4-byte function identifier for dispatch routing
///
/// # Errors
/// * `SelectorCapacityExceeded` - If the registry already has 50 selectors
/// * `SelectorCollision` - If the selector is already registered
/// * `UnauthorizedAccess` - If caller is not the owner
///
/// # Example
/// ```ignore
/// let selector = [0x01, 0x02, 0x03, 0x04];
/// add_module(ctx, facet_program_id, selector)?;
/// ```
pub fn add_module(
    ctx: Context<AddModule>,
    module_address: Pubkey,
    selector: [u8; 4],
) -> Result<()> {
    // Implementation
}
```

### TypeScript Documentation

```typescript
/**
 * Dispatches an instruction to a registered facet through the diamond router.
 * 
 * @param program - The diamond router program instance
 * @param selector - 4-byte function selector
 * @param args - Encoded arguments for the facet
 * @param accounts - Remaining accounts to pass to the facet
 * @returns Transaction signature
 * 
 * @example
 * ```typescript
 * const signature = await dispatchToFacet(
 *   diamondProgram,
 *   [0x01, 0x02, 0x03, 0x04],
 *   encodedArgs,
 *   [treasuryAccount, recipientAccount]
 * );
 * ```
 */
export async function dispatchToFacet(
  program: Program,
  selector: number[],
  args: Buffer,
  accounts: AccountMeta[]
): Promise<string> {
  // Implementation
}
```

---

## 🎯 Areas for Contribution

We especially welcome contributions in these areas:

### High Priority
- [ ] Additional facet examples (DEX, lending, governance)
- [ ] Enhanced testing (fuzzing, property testing)
- [ ] Security hardening (formal verification)
- [ ] Documentation improvements
- [ ] Performance optimizations

### Medium Priority
- [ ] CLI tooling for facet management
- [ ] TypeScript SDK enhancements
- [ ] Migration scripts for state upgrades
- [ ] Monitoring and observability tools
- [ ] Integration with popular Solana libraries

### Future Work
- [ ] Cross-chain bridge facets
- [ ] Advanced governance mechanisms
- [ ] MEV protection strategies
- [ ] L2 integration (Neon, Eclipse)
- [ ] Mobile SDK

---

## 🤔 Questions?

- **General Questions**: [GitHub Discussions](https://github.com/gsknnft/Solana_Diamond_Protocol_dev/discussions)
- **Bug Reports**: [GitHub Issues](https://github.com/gsknnft/Solana_Diamond_Protocol_dev/issues)
- **Feature Requests**: [GitHub Issues](https://github.com/gsknnft/Solana_Diamond_Protocol_dev/issues)
- **Security**: security@sigilnet.io

---

## 🙏 Thank You!

Your contributions help make Solana Diamond Protocol better for everyone. Whether you're fixing a typo, adding a test, or building a new facet, we appreciate your effort!

Happy coding! 🚀
