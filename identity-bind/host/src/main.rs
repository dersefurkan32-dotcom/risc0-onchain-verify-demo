use methods::{IDENTITY_BIND_ELF, IDENTITY_BIND_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

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
