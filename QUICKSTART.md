# Solana Diamond Protocol - Quick Start Guide

## Prerequisites

- Rust 1.87.0 or later
- Solana CLI tools
- Anchor CLI 0.31.1
- Node.js 18+ with npm/pnpm

## Installation

```bash
# Clone the repository
git clone https://github.com/gsknnft/Solana_Diamond_Protocol.git
cd Solana_Diamond_Protocol

# Install dependencies
npm install
# or
pnpm install

# Build the program
cargo build
```

## Building the Diamond Program

```bash
# Build only the diamond program
cd programs/sol_diamond
cargo build

# Or build all workspace members
cd ../..
cargo build --workspace
```

## Running Tests

```bash
# Start local validator (in a separate terminal)
solana-test-validator

# Run tests
anchor test --skip-local-validator

# Or with npm
npm test
```

## Deployment

### 1. Configure Solana CLI

```bash
# Set to devnet for testing
solana config set --url devnet

# Or localnet for development
solana config set --url localhost

# Check your configuration
solana config get
```

### 2. Create and Fund Wallet

```bash
# Generate a new keypair (if needed)
solana-keygen new --outfile ~/.config/solana/id.json

# Get your address
solana address

# Request airdrop (devnet)
solana airdrop 2
```

### 3. Build and Deploy

```bash
# Build the program for deployment
anchor build

# Deploy to configured cluster
anchor deploy

# Note the deployed program ID
```

## Basic Usage

### Initialize Diamond State

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolDiamond } from "./target/types/sol_diamond";

const program = anchor.workspace.SolDiamond as Program<SolDiamond>;
const owner = anchor.web3.Keypair.generate();

// Create diamond state account
const diamondState = anchor.web3.Keypair.generate();

await program.methods
  .initializeDiamond(owner.publicKey, 255)
  .accounts({
    diamondState: diamondState.publicKey,
    payer: owner.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([diamondState, owner])
  .rpc();
```

### Register a Facet Module

```typescript
// Your facet program ID
const facetProgramId = new anchor.web3.PublicKey("YOUR_FACET_PROGRAM_ID");

// Generate a 4-byte selector for the function
// Example: first 4 bytes of SHA256 hash of "transfer"
const selector = [0x01, 0x02, 0x03, 0x04];

await program.methods
  .addModule(facetProgramId, selector)
  .accounts({
    diamondState: diamondState.publicKey,
    authority: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

### Dispatch to a Facet

```typescript
// Prepare instruction data: selector + arguments
const selector = Buffer.from([0x01, 0x02, 0x03, 0x04]);
const args = Buffer.from([...]); // Your facet-specific arguments
const ixData = Buffer.concat([selector, args]);

await program.methods
  .dispatch(Array.from(ixData))
  .accounts({
    routerConfig: diamondState.publicKey,
    module: facetProgramId,
  })
  .remainingAccounts([
    // Pass any accounts your facet needs
    { pubkey: someAccount, isWritable: true, isSigner: false },
    // ...
  ])
  .rpc();
```

### Add an Admin

```typescript
const newAdmin = anchor.web3.Keypair.generate().publicKey;

await program.methods
  .addAdmin(newAdmin)
  .accounts({
    diamondState: diamondState.publicKey,
    owner: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

### View Active Modules

```typescript
// This logs information via msg!() in the program
await program.methods
  .getActiveModules()
  .accounts({
    diamondState: diamondState.publicKey,
  })
  .rpc();

// Or fetch the account directly
const account = await program.account.diamondState.fetch(
  diamondState.publicKey
);
console.log("Active modules:", account.activeModules);
console.log("Admins:", account.admins);
console.log("Selectors:", account.selectors);
```

## Creating a Facet

### 1. Create a New Anchor Program

```bash
anchor init my_facet
cd my_facet
```

### 2. Implement Your Facet Logic

```rust
use anchor_lang::prelude::*;

declare_id!("YourFacetProgramID");

#[program]
pub mod my_facet {
    use super::*;

    // Your facet function
    // Selector will be generated from instruction discriminator
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        // Your logic here
        msg!("Transferring {} tokens", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Signer<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    // Add other required accounts
}
```

### 3. Build and Deploy Your Facet

```bash
anchor build
anchor deploy
```

### 4. Register with Diamond

Use the deployed program ID and calculate the selector (first 4 bytes of the instruction discriminator) to register your facet with the diamond.

## Troubleshooting

### Build Errors

**Problem**: `cargo-features = ["edition2021"]` warning
**Solution**: Remove this line from Cargo.toml (already fixed in latest version)

**Problem**: Stack overflow errors
**Solution**: Use fixed-size arrays instead of Strings, limit Vec sizes (already implemented with MAX_* constants)

**Problem**: `#[program]` macro can't find Context structs
**Solution**: Define all Context structs in lib.rs (already implemented)

### Runtime Errors

**Problem**: "Module not found" when dispatching
**Solution**: Ensure the module is registered with `add_module()` first

**Problem**: "Unauthorized access"
**Solution**: Verify the module pubkey passed matches the registered address

**Problem**: "Max capacity reached"
**Solution**: 
- Admins: Limited to 10
- Modules: Limited to 20  
- Selectors: Limited to 50
Consider using a separate registry PDA for larger deployments

### Testing Issues

**Problem**: Tests timeout
**Solution**: Increase test timeout in Anchor.toml or test command

**Problem**: Account not found
**Solution**: Ensure accounts are properly initialized before use

## Next Steps

1. **Review Architecture**: See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed design
2. **Run Tests**: Execute `anchor test` to validate setup
3. **Deploy to Devnet**: Test in a live environment
4. **Build Facets**: Create your first facet module
5. **Monitor Logs**: Use `solana logs` to debug dispatch calls

## Resources

- [Official Documentation](./ARCHITECTURE.md)
- [EIP-2535 Diamond Standard](https://eips.ethereum.org/EIPS/eip-2535)
- [Anchor Book](https://book.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)

## Support

For issues and questions:
- GitHub Issues: [Create an issue](https://github.com/gsknnft/Solana_Diamond_Protocol/issues)
- Discussions: [GitHub Discussions](https://github.com/gsknnft/Solana_Diamond_Protocol/discussions)

## License

See LICENSE file for details.
