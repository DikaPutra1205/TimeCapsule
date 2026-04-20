#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Helper: create a test environment with a specific ledger timestamp
fn setup_env(timestamp: u64) -> Env {
    let env = Env::default();
    env.mock_all_auths();

    let mut ledger = env.ledger().get();
    ledger.timestamp = timestamp;
    env.ledger().set(ledger);

    env
}

/// Helper: register the contract
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
        &String::from_str(&env, "Letter to 2030"),
        &String::from_str(&env, "Hey future me! Hope you made it!"),
        &String::from_str(&env, "My Future Self"),
        &String::from_str(&env, "future"),
        &3600, // 1 hour from now
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
        &String::from_str(&env, "Capsule 1"),
        &String::from_str(&env, "First message"),
        &String::from_str(&env, "Friend"),
        &String::from_str(&env, "memory"),
        &3600,
    );

    let id2 = client.create_capsule(
        &user,
        &String::from_str(&env, "Capsule 2"),
        &String::from_str(&env, "Second message"),
        &String::from_str(&env, "Family"),
        &String::from_str(&env, "love"),
        &7200,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
#[should_panic(expected = "Unlock delay must be greater than 0 seconds")]
fn test_create_capsule_zero_delay() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Fail"),
        &String::from_str(&env, "Message"),
        &String::from_str(&env, "Anyone"),
        &String::from_str(&env, "test"),
        &0, // Should panic!
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
        &String::from_str(&env, "Secret"),
        &String::from_str(&env, "THIS IS A SECRET MESSAGE!"),
        &String::from_str(&env, "Best Friend"),
        &String::from_str(&env, "secret"),
        &9999, // Still a long time
    );

    let capsules = client.get_capsules();
    assert_eq!(capsules.len(), 1);

    let preview = capsules.get(0).unwrap();
    // Message must be hidden because it's still locked
    assert_eq!(
        preview.message,
        String::from_str(&env, "[LOCKED - Not yet time to open]")
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
        &String::from_str(&env, "Special Capsule"),
        &String::from_str(&env, "Secret content"),
        &String::from_str(&env, "You"),
        &String::from_str(&env, "love"),
        &3600,
    );

    let preview = client.get_capsule(&1);
    assert_eq!(preview.id, 1);
    assert_eq!(preview.title, String::from_str(&env, "Special Capsule"));
    assert_eq!(preview.recipient, String::from_str(&env, "You"));
}

#[test]
fn test_get_my_capsules() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // User1 creates 2 capsules
    client.create_capsule(
        &user1,
        &String::from_str(&env, "From User1 - A"),
        &String::from_str(&env, "Message A"),
        &String::from_str(&env, "Friend"),
        &String::from_str(&env, "memory"),
        &3600,
    );
    client.create_capsule(
        &user1,
        &String::from_str(&env, "From User1 - B"),
        &String::from_str(&env, "Message B"),
        &String::from_str(&env, "Family"),
        &String::from_str(&env, "love"),
        &7200,
    );

    // User2 creates 1 capsule
    client.create_capsule(
        &user2,
        &String::from_str(&env, "From User2"),
        &String::from_str(&env, "Message C"),
        &String::from_str(&env, "Myself"),
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
        &String::from_str(&env, "Message"),
        &String::from_str(&env, "Someone"),
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
        &String::from_str(&env, "Time Capsule"),
        &String::from_str(&env, "Congrats! You successfully opened this capsule!"),
        &String::from_str(&env, "Future Me"),
        &String::from_str(&env, "future"),
        &500, // Unlocks at timestamp 1500
    );

    // Advance time to 2000 (past the 1500 deadline)
    let mut ledger = env.ledger().get();
    ledger.timestamp = 2000;
    env.ledger().set(ledger);

    let message = client.open_capsule(&user, &1);
    assert_eq!(
        message,
        String::from_str(&env, "Congrats! You successfully opened this capsule!")
    );

    // Verify status is now Opened
    let preview = client.get_capsule(&1);
    assert_eq!(preview.is_opened, true);
}

#[test]
#[should_panic(expected = "Capsule is still locked")]
fn test_open_capsule_too_early() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Do Not Open"),
        &String::from_str(&env, "Not yet!"),
        &String::from_str(&env, "Anyone"),
        &String::from_str(&env, "test"),
        &9999, // Unlock far in the future
    );

    // Try to open now — should panic!
    client.open_capsule(&user, &1);
}

#[test]
#[should_panic(expected = "This capsule has already been opened")]
fn test_open_capsule_already_opened() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "One Time Open"),
        &String::from_str(&env, "Read once only"),
        &String::from_str(&env, "Reader"),
        &String::from_str(&env, "test"),
        &100,
    );

    // Advance time
    let mut ledger = env.ledger().get();
    ledger.timestamp = 2000;
    env.ledger().set(ledger);

    // First open — success
    client.open_capsule(&user, &1);

    // Second open — should panic!
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
        &String::from_str(&env, "To Be Deleted"),
        &String::from_str(&env, "This message will disappear"),
        &String::from_str(&env, "Nobody"),
        &String::from_str(&env, "temp"),
        &3600,
    );

    let result = client.delete_capsule(&user, &1);
    assert_eq!(result, String::from_str(&env, "Capsule deleted successfully"));

    // Verify it was deleted
    let capsules = client.get_capsules();
    assert_eq!(capsules.len(), 0);
}

#[test]
#[should_panic(expected = "Only the creator can delete")]
fn test_delete_capsule_not_owner() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.create_capsule(
        &owner,
        &String::from_str(&env, "Owner's Capsule"),
        &String::from_str(&env, "Owner's message"),
        &String::from_str(&env, "Owner"),
        &String::from_str(&env, "private"),
        &3600,
    );

    // Stranger tries to delete — should panic!
    client.delete_capsule(&stranger, &1);
}

#[test]
#[should_panic(expected = "Cannot delete a capsule that has already been opened")]
fn test_delete_opened_capsule() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    client.create_capsule(
        &user,
        &String::from_str(&env, "Already Opened"),
        &String::from_str(&env, "Message"),
        &String::from_str(&env, "Reader"),
        &String::from_str(&env, "test"),
        &100,
    );

    // Advance time & open
    let mut ledger = env.ledger().get();
    ledger.timestamp = 2000;
    env.ledger().set(ledger);
    client.open_capsule(&user, &1);

    // Try to delete an opened capsule — should panic!
    client.delete_capsule(&user, &1);
}

// ========================= TEST: STATS =======================================

#[test]
fn test_get_stats() {
    let env = setup_env(1000);
    let client = create_contract(&env);
    let user = Address::generate(&env);

    // Create 3 capsules with different unlock times
    client.create_capsule(
        &user,
        &String::from_str(&env, "Quick"),
        &String::from_str(&env, "Quick message"),
        &String::from_str(&env, "A"),
        &String::from_str(&env, "test"),
        &100, // Unlocks at 1100
    );
    client.create_capsule(
        &user,
        &String::from_str(&env, "Medium"),
        &String::from_str(&env, "Medium message"),
        &String::from_str(&env, "B"),
        &String::from_str(&env, "test"),
        &5000, // Unlocks at 6000
    );
    client.create_capsule(
        &user,
        &String::from_str(&env, "Long"),
        &String::from_str(&env, "Long message"),
        &String::from_str(&env, "C"),
        &String::from_str(&env, "test"),
        &99999, // Unlocks far in the future
    );

    // Advance time to 1500 (first capsule is now ready)
    let mut ledger = env.ledger().get();
    ledger.timestamp = 1500;
    env.ledger().set(ledger);

    // Open the first capsule
    client.open_capsule(&user, &1);

    let stats = client.get_stats();
    assert_eq!(stats.total_capsules, 3);
    assert_eq!(stats.total_opened, 1);
    assert_eq!(stats.total_locked, 2);  // Capsules 2 & 3 still locked
    assert_eq!(stats.total_ready, 0);   // None ready (1 was opened already)
}
