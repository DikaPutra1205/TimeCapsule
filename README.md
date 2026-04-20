# ⏳ TimeCapsule DApp

**TimeCapsule DApp** — Blockchain-Based Decentralized Digital Time Capsule System

## Project Description

TimeCapsule DApp is a decentralized smart contract solution built on the Stellar blockchain using Soroban SDK. It provides a unique platform for creating **time-locked digital capsules** — users can store secret messages that are sealed and can only be revealed after a specific date and time.

Think of it as a digital time capsule: write a letter to your future self, create a surprise message for a friend, or store an announcement that will be automatically unlocked at the right moment. All stored transparently and immutably on the Stellar blockchain.

The system ensures that messages remain truly hidden until their unlock time, with blockchain-level guarantees that no one — not even the contract deployer — can peek at the contents before the designated time.

## Project Vision

Our vision is to create meaningful human connections across time by:

- **Preserving Memories**: Allowing users to store thoughts, messages, and memories that transcend the present moment
- **Ensuring Time-Lock Integrity**: Leveraging blockchain timestamps to guarantee messages cannot be accessed prematurely
- **Empowering Emotional Connection**: Creating a platform for surprise messages, future letters, and scheduled revelations
- **Building Trust Through Transparency**: Using smart contracts to ensure the rules of time-locking are enforced by code, not promises
- **Decentralizing Personal Data**: Moving sensitive personal messages from centralized servers to a trustless blockchain

## Key Features

### 1. **🔒 Time-Locked Capsule Creation (CREATE)**

- Create capsules with a secret message, title, recipient, and tag
- Set a custom unlock delay (in seconds) for when the capsule can be opened
- Automated unique ID generation for each capsule
- Messages are sealed until unlock time
- Support for categorization via tags (love, future, memory, announcement, etc.)

### 2. **👀 Privacy-Aware Reading (READ)**

- Browse all capsules with automatic message hiding for locked ones
- Locked capsules show `[LOCKED - Not yet time to open]` instead of the actual message
- Ready capsules show `[READY - Open to read the message!]` hint
- Filter capsules by creator address
- Real-time status checking (Locked → Ready → Opened)

### 3. **📖 Capsule Opening (UPDATE)**

- Open capsules that have passed their unlock time
- The secret message is revealed upon opening
- Opening is recorded permanently on the blockchain
- Once opened, the capsule's message becomes publicly visible

### 4. **🗑️ Secure Deletion (DELETE)**

- Remove capsules that haven't been opened yet
- Only the original creator can delete their capsules
- Opened capsules are permanently preserved (cannot be deleted)
- Ownership verification through Soroban authorization

### 5. **📊 Platform Statistics**

- Track total capsules created across the platform
- Monitor how many capsules have been opened vs. still locked
- See how many capsules are ready to be opened
- Real-time stats computed from on-chain data

### 6. **🔐 Authorization & Security**

- All write operations require cryptographic authorization
- Ownership enforcement for deletion
- Time-based access control for message revealing
- Immutable records of all capsule lifecycle events

## Contract Details

- **Network**: Stellar Testnet
- **Contract ID**: `CDUH55HRZMQBHRF75KD3PEPDAOTAG3AD5EMX56ZDFBD3AMLYGLNBHDJQ`

### Testnet Deployment Screenshot

![TimeCapsule Contract on Stellar Testnet](image.png)

## Contract Functions

| Function | Description | Auth Required |
|---|---|---|
| `create_capsule()` | Create a new time-locked capsule | ✅ Creator |
| `get_capsules()` | Get all capsules (messages hidden if locked) | ❌ |
| `get_capsule(id)` | Get a specific capsule by ID | ❌ |
| `get_my_capsules(user)` | Get capsules created by a specific user | ❌ |
| `check_status(id)` | Check if a capsule is Locked/Ready/Opened | ❌ |
| `open_capsule(opener, id)` | Open a ready capsule and reveal the message | ✅ Opener |
| `delete_capsule(creator, id)` | Delete an unopened capsule | ✅ Creator |
| `get_stats()` | Get platform-wide statistics | ❌ |

## Capsule Lifecycle

```
┌──────────┐    time passes    ┌──────────┐    user opens    ┌──────────┐
│  LOCKED  │ ───────────────► │  READY   │ ──────────────► │  OPENED  │
│  🔒      │                  │  🔓      │                  │  📖      │
│ Message  │                  │ Message  │                  │ Message  │
│ Hidden   │                  │ Hidden   │                  │ Revealed │
└──────────┘                  └──────────┘                  └──────────┘
     │                                                           │
     │ creator can DELETE                    cannot be DELETED ───┘
     └──────────────────┐
                        ▼
                   ┌──────────┐
                   │ DELETED  │
                   │  🗑️      │
                   └──────────┘
```

## Use Cases

- 💌 **Love Letters**: Write a message to your partner, set to unlock on your anniversary
- 🎓 **Graduation Messages**: Leave a message for your future graduating self
- 🎂 **Birthday Surprises**: Create a birthday message capsule that unlocks on the exact date
- 📣 **Scheduled Announcements**: Store announcements that automatically become public at launch time
- 📝 **Future Reflections**: Write to your future self and reflect on how much you've grown
- 🏆 **Goal Tracking**: Set a capsule with your goals that unlocks after your target deadline
- 🤝 **Team Messages**: Leave motivational messages for your team's future milestones

## Technical Requirements

- Soroban SDK v25
- Rust programming language
- Stellar blockchain network

## Getting Started

### Build the Contract

```bash
cd contracts/timecapsule
stellar contract build
```

### Run Tests

```bash
cargo test
```

### Deploy to Stellar Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/timecapsule.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

### Interact with the Contract

```bash
# Create a time capsule (unlock in 1 hour = 3600 seconds)
stellar contract invoke \
  --id CDUH55HRZMQBHRF75KD3PEPDAOTAG3AD5EMX56ZDFBD3AMLYGLNBHDJQ \
  --source <SECRET_KEY> \
  --network testnet \
  -- create_capsule \
  --creator <YOUR_ADDRESS> \
  --title "Letter to 2030" \
  --message "Hey future me! Hope you made it!" \
  --recipient "My Future Self" \
  --tag "future" \
  --unlock_delay 3600

# Get all capsules (messages hidden if still locked)
stellar contract invoke \
  --id CDUH55HRZMQBHRF75KD3PEPDAOTAG3AD5EMX56ZDFBD3AMLYGLNBHDJQ \
  --network testnet \
  -- get_capsules

# Open a capsule (only works if unlock time has passed)
stellar contract invoke \
  --id CDUH55HRZMQBHRF75KD3PEPDAOTAG3AD5EMX56ZDFBD3AMLYGLNBHDJQ \
  --source <SECRET_KEY> \
  --network testnet \
  -- open_capsule \
  --opener <YOUR_ADDRESS> \
  --capsule_id 1

# Check capsule status
stellar contract invoke \
  --id CDUH55HRZMQBHRF75KD3PEPDAOTAG3AD5EMX56ZDFBD3AMLYGLNBHDJQ \
  --network testnet \
  -- check_status \
  --capsule_id 1

# Delete a capsule (only creator, only if not opened)
stellar contract invoke \
  --id CDUH55HRZMQBHRF75KD3PEPDAOTAG3AD5EMX56ZDFBD3AMLYGLNBHDJQ \
  --source <SECRET_KEY> \
  --network testnet \
  -- delete_capsule \
  --creator <YOUR_ADDRESS> \
  --capsule_id 1

# Get platform statistics
stellar contract invoke \
  --id CDUH55HRZMQBHRF75KD3PEPDAOTAG3AD5EMX56ZDFBD3AMLYGLNBHDJQ \
  --network testnet \
  -- get_stats
```

## Future Scope

### Short-Term
1. **Capsule Encryption**: End-to-end encryption so only the recipient can read
2. **Multi-Recipient**: Send one capsule to multiple recipients
3. **Rich Content**: Support for images, links, and formatted text in capsules

### Medium-Term
4. **Chain of Capsules**: Create a series of capsules that unlock sequentially
5. **Conditional Unlock**: Unlock based on on-chain events, not just time
6. **Notification Bridge**: Off-chain alerts when a capsule becomes ready

### Long-Term
7. **Cross-Chain Capsules**: Store capsules across multiple blockchains
8. **NFT Integration**: Mint opened capsules as NFT collectibles
9. **DAO Governance**: Community-driven feature development

---

**⏳ TimeCapsule DApp** — *Your words, sealed in time, revealed when the moment is right.*