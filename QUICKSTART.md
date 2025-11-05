# Quick Start Guide

Get the Solana Diamond Protocol MVP running in 5 minutes.

## Step 1: Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor (choose one)
cargo install --git https://github.com/coral-xyz/anchor --tag v0.31.1 anchor-cli
# OR
npm install -g @coral-xyz/anchor-cli
```

## Step 2: Clone and Build

```bash
# Enter the MVP directory
cd Solana_Diamond_Protocol_MVP

# Build Anchor version
anchor build

# Build native version
cd native && ./build.sh && cd ..
```

## Step 3: Start Local Validator

```bash
# In a separate terminal
solana-test-validator
```

## Step 4: Test

```bash
# Test Anchor version
anchor test --skip-local-validator

# Test native version
cd native
cargo test
```

## What's Next?

- Read the [README](README.md) for architecture overview
- Explore `programs/sol_diamond/src/` for Anchor implementation
- Check `native/` for framework-independent version
- See `native/EXAMPLE.md` for usage examples

## Common Issues

### "anchor: command not found"
```bash
cargo install --git https://github.com/coral-xyz/anchor --tag v0.31.1 anchor-cli
```

### "solana: command not found"
```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### Build errors
```bash
# Update Rust
rustup update

# Clean and rebuild
anchor clean
anchor build
```

---

**Ready to explore!** 🚀
