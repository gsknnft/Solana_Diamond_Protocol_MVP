# Solana Diamond Protocol - Flow Diagrams

Visual representations of the diamond architecture patterns and execution flows.

---

## 1. Complete System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLIENT APPLICATION                          │
│  (Web3.js / Anchor Client / Custom SDK)                            │
│                                                                     │
│  Actions:                                                           │
│  • Initialize Diamond                                               │
│  • Register Facets                                                  │
│  • Dispatch Instructions                                            │
│  • Query State                                                      │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             │ Transaction with
                             │ selector + args
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    DIAMOND ROUTER PROGRAM                           │
│                  (sol_diamond - Main Program)                       │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  INSTRUCTION HANDLERS                                         │ │
│  │  • initialize_diamond()      - Set up diamond state          │ │
│  │  • add_module()              - Register new facet            │ │
│  │  • remove_module()           - Unregister facet              │ │
│  │  • dispatch()                - Route to facet via CPI        │ │
│  │  • add_admin()               - Manage administrators         │ │
│  │  • pause_diamond()           - Emergency stop                │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │  DIAMOND STATE (PDA)                                          │ │
│  │  • Owner: Pubkey                                              │ │
│  │  • Admins: Vec<Pubkey> (max 10)                               │ │
│  │  • Active Modules: Vec<ModuleMeta> (max 20)                   │ │
│  │  • Selectors: Vec<SelectorMapping> (max 50)                   │ │
│  │    - selector: [u8; 4] → module: Pubkey                       │ │
│  │  • Emergency State: is_paused, pause_authority                │ │
│  │  • Governance: multisig, realm references                     │ │
│  └───────────────────────────────────────────────────────────────┘ │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             │ CPI (Cross-Program Invocation)
                             │
            ┌────────────────┼────────────────┐
            │                │                │
            ▼                ▼                ▼
┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│  REWARDS       │  │  LP REWARDS    │  │  CUSTOM        │
│  FACET         │  │  FACET         │  │  FACET         │
│                │  │                │  │                │
│  Program ID:   │  │  Program ID:   │  │  Program ID:   │
│  NnP3k6V9...   │  │  3bPDPtgS...   │  │  Your...       │
│                │  │                │  │                │
│  Instructions: │  │  Instructions: │  │  Instructions: │
│  • initialize  │  │  • initialize  │  │  • your_fn_1   │
│  • distribute  │  │  • set_rate    │  │  • your_fn_2   │
│  • claim       │  │  • set_active  │  │  • ...         │
│  • ...         │  │  • ...         │  │                │
└───────┬────────┘  └───────┬────────┘  └───────┬────────┘
        │                   │                   │
        │    Read/Write     │    Read/Write     │
        └───────────────────┼───────────────────┘
                            ▼
            ┌───────────────────────────────┐
            │      SHARED STATE (PDAs)      │
            │                               │
            │  • DiamondState               │
            │    (router registry)          │
            │                               │
            │  • RewardConfig               │
            │    (facet-specific)           │
            │                               │
            │  • LpRewardsConfig            │
            │    (facet-specific)           │
            │                               │
            │  • UserState                  │
            │    (user-specific)            │
            │                               │
            │  All derived via PDA seeds    │
            └───────────────────────────────┘
```

---

## 2. Dispatch Flow (Step-by-Step)

```
Step 1: Client Prepares Instruction
┌──────────────────────────────────────────┐
│  const selector = [0x01, 0x23, 0x45, 0x67]│
│  const args = encodeArgs(amount, recipient)│
│  const ixData = [selector, args]          │
└──────────────────┬───────────────────────┘
                   │
                   ▼
Step 2: Submit to Diamond Router
┌──────────────────────────────────────────┐
│  await diamondProgram.methods            │
│    .dispatch(ixData)                     │
│    .accounts({                           │
│      routerConfig: diamondStatePDA,      │
│      module: rewardsFacetProgramId,      │
│    })                                    │
│    .remainingAccounts([...])             │
│    .rpc()                                │
└──────────────────┬───────────────────────┘
                   │
                   ▼
Step 3: Router Extracts Selector
┌──────────────────────────────────────────┐
│  pub fn dispatch(                        │
│    ctx: Context<Dispatch>,               │
│    ix_data: Vec<u8>                      │
│  ) -> Result<()> {                       │
│    let selector: [u8; 4] =               │
│      ix_data[0..4].try_into()?;          │
└──────────────────┬───────────────────────┘
                   │
                   ▼
Step 4: Router Looks Up Facet
┌──────────────────────────────────────────┐
│    let router_config =                   │
│      &ctx.accounts.router_config;        │
│                                          │
│    let expected_program = router_config  │
│      .get_module_by_selector(selector)   │
│      .ok_or(DiamondError::NotFound)?;    │
└──────────────────┬───────────────────────┘
                   │
                   ▼
Step 5: Router Validates Program ID
┌──────────────────────────────────────────┐
│    let target_program =                  │
│      ctx.accounts.module.key();          │
│                                          │
│    require!(                             │
│      *target_program == expected_program,│
│      DiamondError::UnauthorizedAccess    │
│    );                                    │
└──────────────────┬───────────────────────┘
                   │
                   ▼
Step 6: Router Forwards via CPI
┌──────────────────────────────────────────┐
│    let ix = Instruction {               │
│      program_id: *target_program,        │
│      accounts: ctx.remaining_accounts,   │
│      data: ix_data,                      │
│    };                                    │
│                                          │
│    invoke(&ix, remaining_accounts)?;     │
│    Ok(())                                │
│  }                                       │
└──────────────────┬───────────────────────┘
                   │
                   ▼
Step 7: Facet Executes Logic
┌──────────────────────────────────────────┐
│  Facet receives:                         │
│  • Full instruction data (selector+args) │
│  • All passed accounts                   │
│  • Can read/write shared PDAs            │
│                                          │
│  Returns result to router → client       │
└──────────────────────────────────────────┘
```

---

## 3. Facet Registration Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Owner Deploys New Facet                                    │
│  $ solana program deploy target/deploy/new_facet.so         │
│    --program-id <keypair>                                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Owner Generates Selector for Facet Function               │
│  const selector = generate_selector("distribute_rewards"); │
│  // => [0x01, 0x23, 0x45, 0x67]                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Owner Registers Facet with Diamond                        │
│  await diamondProgram.methods                              │
│    .addModule(newFacetProgramId, selector)                 │
│    .accounts({                                             │
│      diamondState: diamondStatePDA,                        │
│      authority: owner.publicKey,                           │
│    })                                                      │
│    .signers([owner])                                       │
│    .rpc();                                                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Diamond Router Updates State                              │
│  • Adds ModuleMeta to active_modules[]                     │
│  • Adds SelectorMapping to selectors[]                     │
│    - selector: [0x01, 0x23, 0x45, 0x67]                    │
│    - module: newFacetProgramId                             │
│    - function_name: "distribute_rewards"                   │
│  • Emits FacetAdded event                                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Facet is Now Callable via Router                          │
│  Clients can dispatch with selector [0x01, 0x23, 0x45, 0x67]│
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Upgrade Strategy Flow

### A. Facet Upgrade (No Registry Change)

```
┌────────────────────────────────────────────┐
│  Identify Facet Needing Upgrade            │
│  • Bug fix in rewards logic                │
│  • New feature: compound rewards           │
└────────────────┬───────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│  Deploy New Facet Code to SAME Program ID  │
│  $ solana program deploy \                 │
│      target/deploy/rewards_facet_v2.so \   │
│      --program-id NnP3k6V9... \            │
│      --upgrade-authority <keypair>         │
└────────────────┬───────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│  Router Automatically Uses New Version    │
│  • No registry update needed               │
│  • Same program ID, new code               │
│  • Dispatch continues with same selector   │
└────────────────────────────────────────────┘
```

### B. Selector Remapping (Change Target Facet)

```
┌────────────────────────────────────────────┐
│  Deploy New Facet with Better Logic        │
│  • rewards_facet_v2 (new program ID)       │
│  • More efficient, additional features     │
└────────────────┬───────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│  Owner Calls replace_module()              │
│  • Removes old selector mapping            │
│  • Adds new mapping: selector → v2 program │
│  • Emits FacetReplaced event               │
└────────────────┬───────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│  Router Now Dispatches to New Facet        │
│  • Same selector, different program        │
│  • Old facet can be deprecated             │
└────────────────────────────────────────────┘
```

### C. Router Upgrade (With Governance)

```
┌────────────────────────────────────────────┐
│  Propose Router Upgrade                    │
│  • Multisig votes (Squads)                 │
│  • DAO proposal (SPL Governance)           │
└────────────────┬───────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│  Timelock Period (e.g., 7 days)            │
│  • Community reviews change                │
│  • Can veto if malicious                   │
└────────────────┬───────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│  Execute Upgrade After Delay               │
│  $ solana program deploy \                 │
│      target/deploy/sol_diamond_v2.so \     │
│      --program-id DcRCfU3... \             │
│      --upgrade-authority <multisig>        │
└────────────────┬───────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────┐
│  Router Continues with Same State PDA      │
│  • DiamondState persists across upgrades   │
│  • All selector mappings remain valid      │
│  • Facets continue to work                 │
└────────────────────────────────────────────┘
```

---

## 5. State Management Pattern

```
┌────────────────────────────────────────────────────────────┐
│  ROUTER STATE (DiamondState PDA)                           │
│  Seeds: ["diamond_state", owner.key]                       │
│  • Selector registry                                       │
│  • Module metadata                                         │
│  • Admin list                                              │
│  • Emergency pause state                                   │
└────────────────────────────────────────────────────────────┘
                              ▲
                              │
                  ┌───────────┴───────────┐
                  │                       │
┌─────────────────▼──────────┐  ┌─────────▼──────────────────┐
│  FACET STATE (Facet PDA)   │  │  USER STATE (User PDA)     │
│  Seeds: ["rewards_config"] │  │  Seeds: ["user", user.key] │
│  • Reward rate             │  │  • Staked amount           │
│  • Treasury vault          │  │  • Pending rewards         │
│  • Admin authority         │  │  • Last claim time         │
│  • Emission schedule       │  │  • Multiplier              │
└────────────────────────────┘  └────────────────────────────┘
         ▲                               ▲
         │                               │
         └───────────┬───────────────────┘
                     │
      Both accessed by facet during CPI
      Router passes accounts, doesn't interpret
```

---

## 6. Access Control Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│  OWNER (Single Pubkey)                                      │
│  • Initialize diamond                                       │
│  • Add/remove facets                                        │
│  • Manage admins                                            │
│  • Transfer ownership                                       │
│  • Emergency pause                                          │
└────────────────────────┬────────────────────────────────────┘
                         │ delegates to
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  ADMINS (Up to 10 Pubkeys)                                  │
│  • Call administrative facet functions                      │
│  • Update configurations                                    │
│  • Cannot modify router state                               │
└────────────────────────┬────────────────────────────────────┘
                         │ delegates to
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  FACETS (Registered Programs)                               │
│  • Execute business logic                                   │
│  • Read/write facet-specific state                          │
│  • Subject to their own access controls                     │
└────────────────────────┬────────────────────────────────────┘
                         │ serves
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  PUBLIC USERS                                               │
│  • Call registered facet functions                          │
│  • Subject to facet-level validation                        │
│  • Cannot modify router state                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Error Handling Flow

```
Client submits dispatch
        │
        ▼
┌───────────────────┐
│  Router Validates │
└─────┬─────────────┘
      │
      ├─► Selector not found? ──► DiamondError::ModuleNotFound
      │
      ├─► Program ID mismatch? ─► DiamondError::UnauthorizedAccess
      │
      ├─► Diamond paused? ──────► DiamondError::DiamondPaused
      │
      └─► Validation passes
              │
              ▼
      ┌──────────────┐
      │  CPI to Facet│
      └──────┬───────┘
             │
             ├─► Facet validation fails? ──► FacetError::*
             │
             ├─► Facet logic fails? ────────► FacetError::*
             │
             └─► Success
                     │
                     ▼
              Return to client
                  Success!
```

---

## 8. Selector Collision Prevention

```
┌──────────────────────────────────────────────────────────────┐
│  NAMESPACE PARTITIONING (EIP-2535 §5)                        │
│                                                              │
│  Library A                  Library B                        │
│  namespace: hash("libA")    namespace: hash("libB")          │
│  selector: 0x01234567       selector: 0x01234567             │
│  ─────────────────────────► No collision!                    │
│                                                              │
│  SelectorMapping {                                           │
│    namespace: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11],│
│    selector: [0x01, 0x23, 0x45, 0x67],                       │
│    module: facet_a_program_id,                               │
│  }                                                           │
│                                                              │
│  SelectorMapping {                                           │
│    namespace: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],│
│    selector: [0x01, 0x23, 0x45, 0x67],  // Same selector!    │
│    module: facet_b_program_id,          // Different facet   │
│  }                                                           │
└──────────────────────────────────────────────────────────────┘
```

---

## 9. Emergency Pause Mechanism

```
Normal Operation
┌─────────────────┐
│  is_paused: false│
└────────┬─────────┘
         │
         ├─► dispatch() → Success
         │
         └─► add_module() → Success

         ▼ Owner calls pause_diamond()

Emergency State
┌─────────────────┐
│  is_paused: true │
│  pause_reason:   │
│  "Security issue"│
└────────┬─────────┘
         │
         ├─► dispatch() → DiamondError::DiamondPaused
         │
         └─► add_module() → DiamondError::DiamondPaused
                           (Owner can still manage)

         ▼ Owner calls unpause_diamond()

Return to Normal
┌─────────────────┐
│  is_paused: false│
│  resumed_at: ts  │
└─────────────────┘
```

---

## 10. Integration Points

```
                    ┌─────────────────────┐
                    │  EXTERNAL SYSTEMS   │
                    └──────────┬──────────┘
                               │
                ┌──────────────┼──────────────┐
                │              │              │
                ▼              ▼              ▼
        ┌───────────┐  ┌───────────┐  ┌───────────┐
        │  SQUADS   │  │SPL        │  │ SIGILNET  │
        │  MULTISIG │  │GOVERNANCE │  │  BRIDGE   │
        └─────┬─────┘  └─────┬─────┘  └─────┬─────┘
              │              │              │
              │  Set as      │  Set as      │  Custom
              │  upgrade     │  governance  │  facet
              │  authority   │  realm       │
              │              │              │
              └──────────────┼──────────────┘
                             │
                             ▼
                ┌────────────────────────┐
                │  DIAMOND ROUTER        │
                │  governance_program:   │
                │    Some<multisig>      │
                │  governance_realm:     │
                │    Some<realm>         │
                └────────────────────────┘
                             │
                             ▼
                     Dispatches to facets
                     with governance checks
```

---

## Summary

These diagrams illustrate:

1. **System Architecture**: How components interact
2. **Dispatch Flow**: Step-by-step instruction routing
3. **Registration**: How facets join the diamond
4. **Upgrades**: Multiple upgrade strategies
5. **State Management**: PDA usage patterns
6. **Access Control**: Permission hierarchy
7. **Error Handling**: Validation and error paths
8. **Collision Prevention**: Namespace partitioning
9. **Emergency Controls**: Pause mechanism
10. **Integrations**: External system connections

For implementation details, see [README.diamond.md](./README.diamond.md) and [ARCHITECTURE.md](./ARCHITECTURE.md).
