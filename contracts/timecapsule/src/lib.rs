#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec};

// ============================================================================
// ⏳ TIMECAPSULE — Digital Time Capsule on Stellar Blockchain
// ============================================================================
// Platform kapsul waktu digital terdesentralisasi. Pengguna bisa menyimpan
// pesan rahasia yang TERKUNCI WAKTU — pesan baru bisa dibaca setelah tanggal
// tertentu. Cocok untuk: surat untuk masa depan, pengumuman terjadwal,
// hadiah kejutan, atau kenangan yang ingin dibuka nanti.
//
// Fitur utama:
// - Create: Buat kapsul dengan pesan tersembunyi + tanggal buka
// - Read:   Lihat daftar kapsul (pesan tersembunyi jika belum waktunya)
// - Update: Buka kapsul yang sudah melewati waktu unlock
// - Delete: Hapus kapsul milik sendiri (hanya jika belum dibuka)
// ============================================================================

// ========================== DATA STRUCTURES ==================================

/// Kunci penyimpanan kontrak
#[contracttype]
pub enum DataKey {
    Capsules,       // Vec<TimeCapsule> — semua kapsul
    CapsuleCount,   // u64 — counter ID
    TotalOpened,    // u64 — total kapsul yang sudah dibuka
}

/// Status sebuah kapsul waktu
/// 0 = Locked (terkunci, belum waktunya)
/// 1 = Ready (sudah waktunya, belum dibuka)
/// 2 = Opened (sudah dibuka)
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum CapsuleStatus {
    Locked,
    Ready,
    Opened,
}

/// Struktur data utama — sebuah Time Capsule
#[contracttype]
#[derive(Clone, Debug)]
pub struct TimeCapsule {
    pub id: u64,              // ID unik kapsul
    pub creator: Address,     // Alamat pembuat
    pub title: String,        // Judul kapsul
    pub message: String,      // Pesan rahasia (tersembunyi jika locked)
    pub recipient: String,    // Untuk siapa kapsul ini ditujukan
    pub tag: String,          // Kategori (love, future-self, announcement, dll)
    pub created_at: u64,      // Waktu pembuatan (unix timestamp)
    pub unlock_at: u64,       // Waktu kapsul bisa dibuka
    pub is_opened: bool,      // Apakah sudah dibuka
    pub opened_at: u64,       // Waktu dibuka (0 jika belum)
}

/// Versi kapsul yang aman — pesan disembunyikan jika masih terkunci
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
    pub message: String,      // "[LOCKED 🔒]" jika belum waktunya
}

/// Statistik platform
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

    /// Membuat kapsul waktu baru.
    ///
    /// # Arguments
    /// - `creator`       — Alamat pembuat kapsul (harus authorize)
    /// - `title`         — Judul kapsul (misal: "Surat untuk Diriku 2030")
    /// - `message`       — Pesan rahasia yang akan disembunyikan
    /// - `recipient`     — Untuk siapa (misal: "Diri sendiri", "Sahabatku", dll)
    /// - `tag`           — Kategori (misal: "love", "future", "memory")
    /// - `unlock_delay`  — Berapa detik dari sekarang sampai bisa dibuka
    ///
    /// # Returns
    /// ID kapsul yang baru dibuat
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
            panic!("Unlock delay harus lebih dari 0 detik");
        }

        // Generate ID baru
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

        // Simpan ke storage
        let mut capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        capsules.push_back(capsule);

        env.storage().instance().set(&DataKey::Capsules, &capsules);
        env.storage().instance().set(&DataKey::CapsuleCount, &count);

        // Publish event
        env.events().publish(
            (symbol_short!("capsule"), symbol_short!("created")),
            count,
        );

        count
    }

    // ====================== READ =============================================

    /// Mendapatkan semua kapsul (pesan disembunyikan jika masih locked).
    /// Ini adalah fungsi utama untuk menampilkan daftar kapsul.
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

    /// Mendapatkan detail satu kapsul berdasarkan ID.
    /// Pesan hanya ditampilkan jika kapsul sudah dibuka atau sudah waktunya.
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

        panic!("Kapsul tidak ditemukan");
    }

    /// Mendapatkan kapsul-kapsul milik user tertentu
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

    /// Cek status kapsul — apakah sudah bisa dibuka?
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

        panic!("Kapsul tidak ditemukan");
    }

    // ====================== UPDATE (OPEN) ====================================

    /// Membuka kapsul waktu — hanya bisa jika sudah melewati waktu unlock.
    /// Siapapun bisa membuka kapsul yang sudah waktunya.
    ///
    /// # Returns
    /// Pesan rahasia yang tersimpan di dalam kapsul
    pub fn open_capsule(env: Env, opener: Address, capsule_id: u64) -> String {
        opener.require_auth();

        let mut capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();

        for i in 0..capsules.len() {
            let mut capsule = capsules.get(i).unwrap();
            if capsule.id == capsule_id {
                // Validasi
                if capsule.is_opened {
                    panic!("Kapsul ini sudah pernah dibuka");
                }
                if now < capsule.unlock_at {
                    panic!("Kapsul masih terkunci! Belum waktunya dibuka");
                }

                // Buka kapsul
                let revealed_message = capsule.message.clone();
                capsule.is_opened = true;
                capsule.opened_at = now;

                // Update di storage
                capsules.remove(i);
                capsules.insert(i, capsule);
                env.storage().instance().set(&DataKey::Capsules, &capsules);

                // Update statistik
                let total_opened: u64 = env.storage().instance()
                    .get(&DataKey::TotalOpened).unwrap_or(0) + 1;
                env.storage().instance().set(&DataKey::TotalOpened, &total_opened);

                // Publish event
                env.events().publish(
                    (symbol_short!("capsule"), symbol_short!("opened")),
                    capsule_id,
                );

                return revealed_message;
            }
        }

        panic!("Kapsul tidak ditemukan");
    }

    // ====================== DELETE ===========================================

    /// Menghapus kapsul — hanya bisa oleh pembuat, dan hanya jika belum dibuka.
    pub fn delete_capsule(env: Env, creator: Address, capsule_id: u64) -> String {
        creator.require_auth();

        let mut capsules: Vec<TimeCapsule> = env.storage().instance()
            .get(&DataKey::Capsules).unwrap_or(Vec::new(&env));

        for i in 0..capsules.len() {
            let capsule = capsules.get(i).unwrap();
            if capsule.id == capsule_id {
                // Validasi kepemilikan
                if capsule.creator != creator {
                    panic!("Hanya pembuat yang bisa menghapus kapsul ini");
                }
                // Tidak bisa hapus kapsul yang sudah dibuka
                if capsule.is_opened {
                    panic!("Tidak bisa menghapus kapsul yang sudah dibuka");
                }

                // Hapus dari storage
                capsules.remove(i);
                env.storage().instance().set(&DataKey::Capsules, &capsules);

                // Publish event
                env.events().publish(
                    (symbol_short!("capsule"), symbol_short!("deleted")),
                    capsule_id,
                );

                return String::from_str(&env, "Kapsul berhasil dihapus");
            }
        }

        panic!("Kapsul tidak ditemukan");
    }

    // ====================== STATISTICS =======================================

    /// Mendapatkan statistik keseluruhan platform
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

    /// Konversi TimeCapsule ke CapsulePreview (sembunyikan pesan jika locked)
    fn to_preview(env: &Env, capsule: &TimeCapsule, now: u64) -> CapsulePreview {
        let status = Self::get_status(capsule, now);

        // Pesan hanya terlihat jika sudah dibuka (Opened)
        let visible_message = if capsule.is_opened {
            capsule.message.clone()
        } else if now >= capsule.unlock_at {
            // Ready tapi belum dibuka — beri hint
            String::from_str(env, "[READY - Buka untuk membaca pesan!]")
        } else {
            String::from_str(env, "[LOCKED - Belum waktunya dibuka]")
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

    /// Tentukan status kapsul berdasarkan waktu sekarang
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
