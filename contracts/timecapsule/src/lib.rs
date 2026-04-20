#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

// ============================================================================
// ⏳ TIMECAPSULE — Digital Time Capsule on Stellar Blockchain
// ============================================================================
// A decentralized time capsule platform. Users can store secret messages that
// are TIME-LOCKED — messages can only be read after a specific date and time.
// Perfect for: letters to your future self, scheduled announcements,
// surprise gifts, or memories you want to open later.
//
// Core Features (CRUD):
// - Create: Store a capsule with a hidden message + unlock date
// - Read:   View capsule list (messages hidden until unlock time)
// - Update: Open a capsule that has passed its unlock time
// - Delete: Remove your own capsule (only if it hasn't been opened)
// ============================================================================

// ========================== DATA STRUCTURES ==================================

/// Contract storage keys
#[contracttype]
pub enum DataKey {
    Capsules,       // Vec<TimeCapsule> — all capsules
    CapsuleCount,   // u64 — ID counter
    TotalOpened,    // u64 — total capsules that have been opened
}

/// Status of a time capsule
/// Locked = not yet time to open
/// Ready  = unlock time has passed, waiting to be opened
/// Opened = has been opened, message revealed
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum CapsuleStatus {
    Locked,
    Ready,
    Opened,
}

/// Main data structure — a Time Capsule
#[contracttype]
#[derive(Clone, Debug)]
pub struct TimeCapsule {
    pub id: u64,              // Unique capsule ID
    pub creator: Address,     // Creator's address
    pub title: String,        // Capsule title
    pub message: String,      // Secret message (hidden when locked)
    pub recipient: String,    // Who this capsule is dedicated to
    pub tag: String,          // Category (love, future-self, announcement, etc.)
    pub created_at: u64,      // Creation timestamp (unix)
    pub unlock_at: u64,       // When the capsule can be opened
    pub is_opened: bool,      // Whether it has been opened
    pub opened_at: u64,       // When it was opened (0 if not yet)
}

/// Safe preview version — message is hidden if still locked
#[contracttype]
#[derive(Clone, Debug)]
pub struct CapsulePreview {
    pub id: u64,
    pub creator: Address,
    pub title: String,
    pub recipient: String,
    pub tag: String,
    pub created_at: u64,
    pub unlock_at: u64,
    pub is_opened: bool,
    pub opened_at: u64,
    pub status: CapsuleStatus,
    pub message: String,      // Shows "[LOCKED]" if not yet time
}

/// Platform-wide statistics
#[contracttype]
#[derive(Clone, Debug)]
pub struct Stats {
    pub total_capsules: u64,
    pub total_opened: u64,
    pub total_locked: u64,
    pub total_ready: u64,
}

// ============================ CONTRACT =======================================

#[contract]
pub struct TimeCapsuleContract;

#[contractimpl]
impl TimeCapsuleContract {

    // ====================== CREATE ===========================================

    /// Create a new time capsule.
    ///
    /// # Arguments
    /// - `creator`       — Creator's address (must authorize)
    /// - `title`         — Capsule title (e.g., "Letter to Myself in 2030")
    /// - `message`       — The secret message to be hidden
    /// - `recipient`     — Who it's for (e.g., "My Future Self", "Best Friend")
    /// - `tag`           — Category (e.g., "love", "future", "memory")
    /// - `unlock_delay`  — Seconds from now until the capsule can be opened
    ///
    /// # Returns
    /// The ID of the newly created capsule
    pub fn create_capsule(
        env: Env,
        creator: Address,
        title: String,
        message: String,
        recipient: String,
        tag: String,
        unlock_delay: u64,
    ) -> u64 {
        creator.require_auth();

        if unlock_delay == 0 {
            panic!("Unlock delay must be greater than 0 seconds");
        }

        // Generate new ID
        let mut count: u64 = env.storage().instance()
            .get(&DataKey::CapsuleCount).unwrap_or(0);
        count += 1;

        let now = env.ledger().timestamp();

        let capsule = TimeCapsule {
            id: count,
            creator: creator.clone(),
            title,
            message,
            recipient,
            tag,
            created_at: now,
            unlock_at: now + unlock_delay,
            is_opened: false,
            opened_at: 0,
        };

        // Save to storage
        let mut capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        capsules.push_back(capsule);

        env.storage().instance().set(&DataKey::Capsules, &capsules);
        env.storage().instance().set(&DataKey::CapsuleCount, &count);

        count
    }

    // ====================== READ =============================================

    /// Get all capsules (messages are hidden if still locked).
    /// This is the main function to display the capsule list.
    pub fn get_capsules(env: Env) -> Vec<CapsulePreview> {
        let capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();
        let mut previews = Vec::new(&env);

        for i in 0..capsules.len() {
            let capsule = capsules.get(i).unwrap();
            let preview = Self::to_preview(&env, &capsule, now);
            previews.push_back(preview);
        }

        previews
    }

    /// Get details of a single capsule by ID.
    /// The message is only shown if the capsule has been opened.
    pub fn get_capsule(env: Env, capsule_id: u64) -> CapsulePreview {
        let capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();

        for i in 0..capsules.len() {
            let capsule = capsules.get(i).unwrap();
            if capsule.id == capsule_id {
                return Self::to_preview(&env, &capsule, now);
            }
        }

        panic!("Capsule not found");
    }

    /// Get all capsules created by a specific user
    pub fn get_my_capsules(env: Env, user: Address) -> Vec<CapsulePreview> {
        let capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();
        let mut result = Vec::new(&env);

        for i in 0..capsules.len() {
            let capsule = capsules.get(i).unwrap();
            if capsule.creator == user {
                result.push_back(Self::to_preview(&env, &capsule, now));
            }
        }

        result
    }

    /// Check the status of a capsule — is it ready to be opened?
    pub fn check_status(env: Env, capsule_id: u64) -> CapsuleStatus {
        let capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();

        for i in 0..capsules.len() {
            let capsule = capsules.get(i).unwrap();
            if capsule.id == capsule_id {
                return Self::get_status(&capsule, now);
            }
        }

        panic!("Capsule not found");
    }

    // ====================== UPDATE (OPEN) ====================================

    /// Open a time capsule — only works if the unlock time has passed.
    /// Anyone can open a capsule that is ready.
    ///
    /// # Returns
    /// The secret message stored inside the capsule
    pub fn open_capsule(env: Env, opener: Address, capsule_id: u64) -> String {
        opener.require_auth();

        let mut capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();

        for i in 0..capsules.len() {
            let mut capsule = capsules.get(i).unwrap();
            if capsule.id == capsule_id {
                // Validation
                if capsule.is_opened {
                    panic!("This capsule has already been opened");
                }
                if now < capsule.unlock_at {
                    panic!("Capsule is still locked! It is not time to open yet");
                }

                // Open the capsule
                let revealed_message = capsule.message.clone();
                capsule.is_opened = true;
                capsule.opened_at = now;

                // Update in storage
                capsules.remove(i);
                capsules.insert(i, capsule);
                env.storage().instance().set(&DataKey::Capsules, &capsules);

                // Update statistics
                let total_opened: u64 = env.storage().instance()
                    .get(&DataKey::TotalOpened).unwrap_or(0) + 1;
                env.storage().instance().set(&DataKey::TotalOpened, &total_opened);

                return revealed_message;
            }
        }

        panic!("Capsule not found");
    }

    // ====================== DELETE ===========================================

    /// Delete a capsule — only the creator can delete, and only if not yet opened.
    pub fn delete_capsule(env: Env, creator: Address, capsule_id: u64) -> String {
        creator.require_auth();

        let mut capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));

        for i in 0..capsules.len() {
            let capsule = capsules.get(i).unwrap();
            if capsule.id == capsule_id {
                // Verify ownership
                if capsule.creator != creator {
                    panic!("Only the creator can delete this capsule");
                }
                // Cannot delete an opened capsule
                if capsule.is_opened {
                    panic!("Cannot delete a capsule that has already been opened");
                }

                // Remove from storage
                capsules.remove(i);
                env.storage().instance().set(&DataKey::Capsules, &capsules);

                return String::from_str(&env, "Capsule deleted successfully");
            }
        }

        panic!("Capsule not found");
    }

    // ====================== STATISTICS =======================================

    /// Get platform-wide statistics
    pub fn get_stats(env: Env) -> Stats {
        let capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();

        let mut locked: u64 = 0;
        let mut ready: u64 = 0;
        let total_opened: u64 = env.storage().instance()
            .get(&DataKey::TotalOpened).unwrap_or(0);

        for i in 0..capsules.len() {
            let capsule = capsules.get(i).unwrap();
            if !capsule.is_opened {
                if now >= capsule.unlock_at {
                    ready += 1;
                } else {
                    locked += 1;
                }
            }
        }

        Stats {
            total_capsules: capsules.len() as u64 + total_opened,
            total_opened,
            total_locked: locked,
            total_ready: ready,
        }
    }

    // ====================== HELPER (INTERNAL) ================================

    /// Convert a TimeCapsule to CapsulePreview (hides message if locked)
    fn to_preview(env: &Env, capsule: &TimeCapsule, now: u64) -> CapsulePreview {
        let status = Self::get_status(capsule, now);

        // Message is only visible if the capsule has been opened
        let visible_message = if capsule.is_opened {
            capsule.message.clone()
        } else if now >= capsule.unlock_at {
            // Ready but not yet opened — show hint
            String::from_str(env, "[READY - Open to read the message!]")
        } else {
            String::from_str(env, "[LOCKED - Not yet time to open]")
        };

        CapsulePreview {
            id: capsule.id,
            creator: capsule.creator.clone(),
            title: capsule.title.clone(),
            recipient: capsule.recipient.clone(),
            tag: capsule.tag.clone(),
            created_at: capsule.created_at,
            unlock_at: capsule.unlock_at,
            is_opened: capsule.is_opened,
            opened_at: capsule.opened_at,
            status,
            message: visible_message,
        }
    }

    /// Determine capsule status based on current time
    fn get_status(capsule: &TimeCapsule, now: u64) -> CapsuleStatus {
        if capsule.is_opened {
            CapsuleStatus::Opened
        } else if now >= capsule.unlock_at {
            CapsuleStatus::Ready
        } else {
            CapsuleStatus::Locked
        }
    }
}

mod test;
