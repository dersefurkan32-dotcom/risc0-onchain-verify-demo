# identity-bind

RISC Zero guest + host for the on-chain verify lab.

The guest commits `(receipt_id, owner_id, amount)` only when `owner_id == expected_owner` and `0 < amount <= pledged`.

```bash
RISC0_DEV_MODE=1 cargo run --release -- --help
```

`--groth16 --save PATH` writes a verifier-ready receipt. Regenerating it requires `rzup` and Docker; the parent repo already commits `receipt-groth16.json`.
