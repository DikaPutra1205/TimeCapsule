#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Helper: buat environment test dengan ledger timestamp tertentu
fn setup_env(timestamp: u64) -> Env {
    let env = Env::default();
    env.mock_all_auths();

    let mut ledger = env.ledger().get();
    ledger.timestamp = timestamp;
    env.ledger().set(ledger);

    env
}

/// Helper: register contract
fn create_contract(env: &Env) -> TimeCapsuleContractClient {
    TimeCapsuleContractClient::new(env, &env.register(TimeCapsuleContract, ()))
}

// ========================= TEST: CREATE ======================================

#[test]
fn test_create_capsule() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    let id = client.create_capsule(
        &user,
        &String::from_str(&env, "Surat untuk 2030"),
        &String::from_str(&env, "Hai masa depan! Semoga kamu sukses!"),
        &String::from_str(&env, "Diri sendiri"),
        &String::from_str(&env, "future"),
        &3600, // 1 jam dari sekarang
    );

    assert_eq!(id, 1);
}

#[test]
fn test_create_multiple_capsules() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    let id1 = client.create_capsule(
        &user,
        &String::from_str(&env, "Kapsul 1"),
        &String::from_str(&env, "Pesan pertama"),
        &String::from_str(&env, "Teman"),
        &String::from_str(&env, "memory"),
        &3600,
    );

    let id2 = client.create_capsule(
        &user,
        &String::from_str(&env, "Kapsul 2"),
        &String::from_str(&env, "Pesan kedua"),
        &String::from_str(&env, "Keluarga"),
        &String::from_str(&env, "love"),
        &7200,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
#[should_panic(expected = "Unlock delay harus lebih dari 0 detik")]
fn test_create_capsule_zero_delay() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Gagal"),
        &String::from_str(&env, "Pesan"),
        &String::from_str(&env, "Siapapun"),
        &String::from_str(&env, "test"),
        &0, // Harus panic!
    );
}

// ========================= TEST: READ ========================================

#[test]
fn test_get_capsules_message_hidden_when_locked() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Rahasia"),
        &String::from_str(&env, "INI PESAN RAHASIA!"),
        &String::from_str(&env, "Sahabat"),
        &String::from_str(&env, "secret"),
        &9999, // Masih lama
    );

    let capsules = client.get_capsules();
    assert_eq!(capsules.len(), 1);

    let preview = capsules.get(0).unwrap();
    // Pesan harus tersembunyi karena masih locked
    assert_eq!(
        preview.message,
        String::from_str(&env, "[LOCKED - Belum waktunya dibuka]")
    );
    assert_eq!(preview.is_opened, false);
}

#[test]
fn test_get_capsule_by_id() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Kapsul Spesial"),
        &String::from_str(&env, "Isi rahasia"),
        &String::from_str(&env, "Kamu"),
        &String::from_str(&env, "love"),
        &3600,
    );

    let preview = client.get_capsule(&1);
    assert_eq!(preview.id, 1);
    assert_eq!(preview.title, String::from_str(&env, "Kapsul Spesial"));
    assert_eq!(preview.recipient, String::from_str(&env, "Kamu"));
}

#[test]
fn test_get_my_capsules() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // User1 buat 2 kapsul
    client.create_capsule(
        &user1,
        &String::from_str(&env, "Dari User1 - A"),
        &String::from_str(&env, "Pesan A"),
        &String::from_str(&env, "Teman"),
        &String::from_str(&env, "memory"),
        &3600,
    );
    client.create_capsule(
        &user1,
        &String::from_str(&env, "Dari User1 - B"),
        &String::from_str(&env, "Pesan B"),
        &String::from_str(&env, "Keluarga"),
        &String::from_str(&env, "love"),
        &7200,
    );

    // User2 buat 1 kapsul
    client.create_capsule(
        &user2,
        &String::from_str(&env, "Dari User2"),
        &String::from_str(&env, "Pesan C"),
        &String::from_str(&env, "Diri sendiri"),
        &String::from_str(&env, "future"),
        &1800,
    );

    let user1_capsules = client.get_my_capsules(&user1);
    let user2_capsules = client.get_my_capsules(&user2);

    assert_eq!(user1_capsules.len(), 2);
    assert_eq!(user2_capsules.len(), 1);
}

#[test]
fn test_check_status_locked() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Locked"),
        &String::from_str(&env, "Pesan"),
        &String::from_str(&env, "Siapa"),
        &String::from_str(&env, "test"),
        &5000,
    );

    let status = client.check_status(&1);
    assert_eq!(status, CapsuleStatus::Locked);
}

// ========================= TEST: OPEN ========================================

#[test]
fn test_open_capsule_success() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Kapsul Waktu"),
        &String::from_str(&env, "Selamat! Kamu berhasil membuka kapsul ini!"),
        &String::from_str(&env, "Masa Depan"),
        &String::from_str(&env, "future"),
        &500, // unlock di timestamp 1500
    );

    // Majukan waktu ke 2000 (sudah lewat deadline 1500)
    let mut ledger = env.ledger().get();
    ledger.timestamp = 2000;
    env.ledger().set(ledger);

    let message = client.open_capsule(&user, &1);
    assert_eq!(
        message,
        String::from_str(&env, "Selamat! Kamu berhasil membuka kapsul ini!")
    );

    // Verifikasi status sudah Opened
    let preview = client.get_capsule(&1);
    assert_eq!(preview.is_opened, true);
}

#[test]
#[should_panic(expected = "Kapsul masih terkunci")]
fn test_open_capsule_too_early() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Jangan Dibuka"),
        &String::from_str(&env, "Belum waktunya!"),
        &String::from_str(&env, "Siapapun"),
        &String::from_str(&env, "test"),
        &9999, // Unlock jauh di masa depan
    );

    // Coba buka sekarang — harus panic!
    client.open_capsule(&user, &1);
}

#[test]
#[should_panic(expected = "Kapsul ini sudah pernah dibuka")]
fn test_open_capsule_already_opened() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Sekali Buka"),
        &String::from_str(&env, "Pesan sekali baca"),
        &String::from_str(&env, "Pembaca"),
        &String::from_str(&env, "test"),
        &100,
    );

    // Majukan waktu
    let mut ledger = env.ledger().get();
    ledger.timestamp = 2000;
    env.ledger().set(ledger);

    // Buka pertama — sukses
    client.open_capsule(&user, &1);

    // Buka kedua — harus panic!
    client.open_capsule(&user, &1);
}

// ========================= TEST: DELETE ======================================

#[test]
fn test_delete_capsule_success() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Akan Dihapus"),
        &String::from_str(&env, "Pesan ini akan hilang"),
        &String::from_str(&env, "Nobody"),
        &String::from_str(&env, "temp"),
        &3600,
    );

    let result = client.delete_capsule(&user, &1);
    assert_eq!(result, String::from_str(&env, "Kapsul berhasil dihapus"));

    // Verifikasi sudah terhapus
    let capsules = client.get_capsules();
    assert_eq!(capsules.len(), 0);
}

#[test]
#[should_panic(expected = "Hanya pembuat yang bisa menghapus")]
fn test_delete_capsule_not_owner() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.create_capsule(
        &owner,
        &String::from_str(&env, "Milik Owner"),
        &String::from_str(&env, "Pesan owner"),
        &String::from_str(&env, "Owner"),
        &String::from_str(&env, "private"),
        &3600,
    );

    // Stranger coba hapus — harus panic!
    client.delete_capsule(&stranger, &1);
}

#[test]
#[should_panic(expected = "Tidak bisa menghapus kapsul yang sudah dibuka")]
fn test_delete_opened_capsule() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Sudah Dibuka"),
        &String::from_str(&env, "Pesan"),
        &String::from_str(&env, "Reader"),
        &String::from_str(&env, "test"),
        &100,
    );

    // Majukan waktu & buka
    let mut ledger = env.ledger().get();
    ledger.timestamp = 2000;
    env.ledger().set(ledger);
    client.open_capsule(&user, &1);

    // Coba hapus yang sudah dibuka — harus panic!
    client.delete_capsule(&user, &1);
}

// ========================= TEST: STATS =======================================

#[test]
fn test_get_stats() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    // Buat 3 kapsul dengan waktu unlock berbeda
    client.create_capsule(
        &user,
        &String::from_str(&env, "Cepat"),
        &String::from_str(&env, "Pesan cepat"),
        &String::from_str(&env, "A"),
        &String::from_str(&env, "test"),
        &100, // Unlock di 1100
    );
    client.create_capsule(
        &user,
        &String::from_str(&env, "Sedang"),
        &String::from_str(&env, "Pesan sedang"),
        &String::from_str(&env, "B"),
        &String::from_str(&env, "test"),
        &5000, // Unlock di 6000
    );
    client.create_capsule(
        &user,
        &String::from_str(&env, "Lama"),
        &String::from_str(&env, "Pesan lama"),
        &String::from_str(&env, "C"),
        &String::from_str(&env, "test"),
        &99999, // Unlock jauh
    );

    // Majukan waktu ke 1500 (kapsul pertama sudah ready)
    let mut ledger = env.ledger().get();
    ledger.timestamp = 1500;
    env.ledger().set(ledger);

    // Buka kapsul pertama
    client.open_capsule(&user, &1);

    let stats = client.get_stats();
    assert_eq!(stats.total_capsules, 3);
    assert_eq!(stats.total_opened, 1);
    assert_eq!(stats.total_locked, 2);  // kapsul 2 & 3 masih locked
    assert_eq!(stats.total_ready, 0);   // tidak ada yang ready (1 sudah dibuka)
}
