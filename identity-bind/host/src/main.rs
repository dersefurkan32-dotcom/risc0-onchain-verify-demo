use methods::{IDENTITY_BIND_ELF, IDENTITY_BIND_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};

fn print_help() {
    eprintln!(
        "identity-bind host\n\
         \n\
         Usage:\n\
           cargo run --release -- [flags]\n\
         \n\
         Flags:\n\
           --help       this message\n\
           --groth16    Groth16 seal (needed for the Solidity verifier)\n\
           --save PATH  write the receipt JSON to PATH\n\
           --mismatch   expected owner != owner (guest panics, no receipt)\n\
           --overdraw   amount > pledged (guest panics, no receipt)\n\
         \n\
         Dev mode (no real proof):\n\
           RISC0_DEV_MODE=1 cargo run --release"
    );
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    if std::env::args().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    let mismatch = std::env::args().any(|a| a == "--mismatch");
    let overdraw = std::env::args().any(|a| a == "--overdraw");
    let groth16 = std::env::args().any(|a| a == "--groth16");
    let save_path = std::env::args()
        .position(|a| a == "--save")
        .and_then(|i| std::env::args().nth(i + 1));

    let receipt_id: u64 = 7;
    let owner_id: u64 = 0xA11CE;
    let pledged: u64 = 1_000;
    let amount: u64 = if overdraw { pledged + 1 } else { 100 };
    let expected_owner: u64 = if mismatch { owner_id.wrapping_add(1) } else { owner_id };

    eprintln!(
        "proving identity-bind receipt_id={receipt_id} owner={owner_id:#x} amount={amount} pledged={pledged} expected={expected_owner:#x}"
    );

    let env = ExecutorEnv::builder()
        .write(&receipt_id)
        .unwrap()
        .write(&owner_id)
        .unwrap()
        .write(&amount)
        .unwrap()
        .write(&expected_owner)
        .unwrap()
        .write(&pledged)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = if groth16 {
        // On-chain-verifyable receipt: Groth16 seal instead of the default
        // succinct one. Needed for the risc0-ethereum verifier (lab 8-14).
        prover
            .prove_with_opts(env, IDENTITY_BIND_ELF, &ProverOpts::groth16())
            .unwrap()
    } else {
        prover.prove(env, IDENTITY_BIND_ELF).unwrap()
    };
    let receipt = prove_info.receipt;

    let (out_id, out_owner, out_amount): (u64, u64, u64) = receipt.journal.decode().unwrap();
    receipt.verify(IDENTITY_BIND_ID).unwrap();

    println!("verified image_id ok");
    println!("journal receipt_id={out_id} owner={out_owner:#x} amount={out_amount}");

    if let Some(path) = save_path {
        let bytes = serde_json::to_vec(&receipt).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        println!("receipt saved to {path} ({} bytes)", bytes.len());
    }
}

/// Guest-side tests: execute the program natively (no proving) and check the
/// security rules that live inside the proof. These run in seconds and need
/// no Groth16 pipeline: `cargo test` from `identity-bind/`.
#[cfg(test)]
mod tests {
    use methods::IDENTITY_BIND_ELF;
    use risc0_zkvm::{default_executor, ExecutorEnv};

    fn execute(
        receipt_id: u64,
        owner_id: u64,
        amount: u64,
        expected_owner: u64,
        pledged: u64,
    ) -> Result<risc0_zkvm::SessionInfo, String> {
        let env = ExecutorEnv::builder()
            .write(&receipt_id)
            .unwrap()
            .write(&owner_id)
            .unwrap()
            .write(&amount)
            .unwrap()
            .write(&expected_owner)
            .unwrap()
            .write(&pledged)
            .unwrap()
            .build()
            .unwrap();
        default_executor()
            .execute(env, IDENTITY_BIND_ELF)
            .map_err(|e| format!("{e:?}"))
    }

    /// The honest case: execution succeeds and the journal commits exactly
    /// (receipt_id, owner, amount) — nothing more, nothing else.
    #[test]
    fn honest_withdrawal_executes_and_commits_journal() {
        let info = execute(7, 0xA11CE, 100, 0xA11CE, 1_000).unwrap();
        let (id, owner, amount): (u64, u64, u64) = info.journal.decode().unwrap();
        assert_eq!((id, owner, amount), (7, 0xA11CE, 100));
    }

    /// Identity split: owner != expected owner -> guest panics -> no receipt
    /// can exist for this statement.
    #[test]
    fn mismatched_identity_fails_execution() {
        assert!(execute(7, 0xA11CE, 100, 0xA11CF, 1_000).is_err());
    }

    /// Collateral rule: amount > pledged -> guest panics, even with the
    /// correct identity.
    #[test]
    fn overdraw_fails_execution() {
        assert!(execute(7, 0xA11CE, 1_001, 0xA11CE, 1_000).is_err());
    }

    /// Boundary: amount == pledged is allowed, pledged + 1 is not.
    #[test]
    fn boundary_at_pledged() {
        assert!(execute(7, 0xA11CE, 1_000, 0xA11CE, 1_000).is_ok());
        assert!(execute(7, 0xA11CE, 1_001, 0xA11CE, 1_000).is_err());
    }

    /// Zero amount is rejected: a receipt must move something.
    #[test]
    fn zero_amount_fails_execution() {
        assert!(execute(7, 0xA11CE, 0, 0xA11CE, 1_000).is_err());
    }
}
