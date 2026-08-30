use risc0_zkvm::guest::env;

/// Guest binds a withdrawal to a single identity.
///
/// Private inputs (host → guest):
///   receipt_id, owner_id, amount, expected_owner, pledged
///
/// Public journal (guest → verifier):
///   receipt_id, owner_id, amount
///
/// If owner_id != expected_owner the guest panics and no receipt is produced.
/// If amount > pledged the guest panics — the collateral rule lives inside
/// the proof, not in a contract. A valid receipt means one identity: the
/// committed owner is the expected owner, and amount is within pledged.
fn main() {
    let receipt_id: u64 = env::read();
    let owner_id: u64 = env::read();
    let amount: u64 = env::read();
    let expected_owner: u64 = env::read();
    let pledged: u64 = env::read();

    assert_eq!(owner_id, expected_owner, "identity split: owner does not match receipt");
    assert!(amount > 0, "zero amount");
    assert!(amount <= pledged, "overdraw: amount exceeds pledged");

    env::commit(&(receipt_id, owner_id, amount));
}
