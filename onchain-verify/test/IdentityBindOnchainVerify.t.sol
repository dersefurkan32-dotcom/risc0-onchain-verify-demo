// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {
    RiscZeroGroth16Verifier
} from "risc0-ethereum/contracts/src/groth16/RiscZeroGroth16Verifier.sol";
import {ControlID} from "risc0-ethereum/contracts/src/groth16/ControlID.sol";

/// Verify a real RISC Zero Groth16 receipt on a local EVM.
/// Receipt produced by the identity-bind guest (risc0-zkvm 3.0.6).
/// journal = (receipt_id=7, owner=0xA11CE, amount=100) as three little-endian u64.
contract IdentityBindOnchainVerifyTest is Test {
    bytes32 constant IMAGE_ID = 0xeb8c0bba0591976ce263f7ff19b5da79b08f5a303362c796059390d121eded11;

    bytes constant SEAL = // 256 bytes, raw Groth16 seal (no selector prefix)
         hex"2dee8dcb0d61ac1af4052361be53fdcce64157a99c9a6242f71b79b7d91ef4f2"
        hex"210ea7dba41081d0b8496b03b441229581704a072f6c9e804837da814dfb6041"
        hex"122fdd70799e8c73c5ccd346eb4d25f1fad3a14e11c90a5b3396bce247b544fa"
        hex"0df68a9381ee46ed14303d3e08a08029c3f5378342bd3706c158dab08ca13f5d"
        hex"0404f8851a88dc3ae3a8d379281e648f40ebefacff3f05a14b9105de374acef1"
        hex"123c6a53462bc447ef8e2105c29efa31cbdebd8b603a2c3a6cc3e3719ecdbeab"
        hex"131bf969daebf8b9b346872accaf9c04b08357b1a6b98a5c1a7327383a135fff"
        hex"1872433809fadd2b165302f3a870e804ea6d1e99c4c6bc4867eb503ec0742157";

    bytes constant JOURNAL = // 24 bytes
        hex"0700000000000000ce110a00000000006400000000000000";

    RiscZeroGroth16Verifier internal verifier;

    function setUp() public {
        verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ROOT, ControlID.BN254_CONTROL_ID);
    }

    function test_groth16_receipt_verifies_onchain() public view {
        bytes memory fullSeal = abi.encodePacked(verifier.SELECTOR(), SEAL);
        verifier.verify(fullSeal, IMAGE_ID, sha256(JOURNAL));
    }

    function test_tampered_journal_reverts() public {
        bytes memory tampered = new bytes(JOURNAL.length);
        for (uint256 i = 0; i < JOURNAL.length; i++) {
            tampered[i] = JOURNAL[i];
        }
        tampered[16] = bytes1(uint8(101)); // amount 100 -> 101
        bytes memory fullSeal = abi.encodePacked(verifier.SELECTOR(), SEAL);
        // sha256 MUST be computed before expectRevert: the builtin compiles to a
        // precompile staticcall, and expectRevert would swallow it as "the call".
        bytes32 tamperedDigest = sha256(tampered);
        vm.expectRevert();
        verifier.verify(fullSeal, IMAGE_ID, tamperedDigest);
    }

    function test_wrong_image_id_reverts() public {
        bytes memory fullSeal = abi.encodePacked(verifier.SELECTOR(), SEAL);
        bytes32 journalDigest = sha256(JOURNAL);
        vm.expectRevert();
        verifier.verify(fullSeal, bytes32(uint256(1)), journalDigest);
    }
}
