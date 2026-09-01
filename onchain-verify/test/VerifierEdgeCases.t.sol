// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IdentityBindOnchainVerifyTest} from "./IdentityBindOnchainVerify.t.sol";

/// Edge and adversarial cases around the Groth16 verifier binding.
/// Inherits the committed receipt, image ID and verifier from the base suite.
contract VerifierEdgeCasesTest is IdentityBindOnchainVerifyTest {
    /// The raw verifier is stateless: the same receipt verifies twice.
    /// That is correct at this layer — replay resistance is the application's
    /// job (bind chain id / context / nonce inside the journal). This test
    /// pins the behavior so reviewers see it instead of assuming it.
    function test_raw_verifier_accepts_replay_by_design() public view {
        bytes memory fullSeal = abi.encodePacked(verifier.SELECTOR(), SEAL);
        bytes32 journalDigest = sha256(JOURNAL);
        verifier.verify(fullSeal, IMAGE_ID, journalDigest);
        verifier.verify(fullSeal, IMAGE_ID, journalDigest); // still valid: no nonce consumed
    }

    /// The selector binds the verifier version. A foreign selector reverts.
    function test_wrong_selector_prefix_reverts() public {
        bytes memory badSeal = abi.encodePacked(bytes4(0xdeadbeef), SEAL);
        bytes32 journalDigest = sha256(JOURNAL);
        vm.expectRevert();
        verifier.verify(badSeal, IMAGE_ID, journalDigest);
    }

    function test_truncated_seal_reverts() public {
        bytes memory short = new bytes(SEAL.length - 1);
        for (uint256 i = 0; i < short.length; i++) {
            short[i] = SEAL[i];
        }
        bytes memory fullSeal = abi.encodePacked(verifier.SELECTOR(), short);
        bytes32 journalDigest = sha256(JOURNAL);
        vm.expectRevert();
        verifier.verify(fullSeal, IMAGE_ID, journalDigest);
    }

    /// Seal malleability: the verifier decodes a fixed-size Groth16 seal and
    /// ignores trailing bytes, so `seal ‖ 0x00` verifies too. The decoded
    /// statement is unchanged — the same (image ID, journal) pair is proven —
    /// so this is not a forgery. But any integration that keys a nullifier or
    /// a replay registry on RAW SEAL BYTES would see two "different" receipts.
    /// Key on (imageId, journalDigest) instead. This test pins the behavior.
    function test_appended_garbage_accepted_seal_malleability() public view {
        bytes32 journalDigest = sha256(JOURNAL);
        verifier.verify(abi.encodePacked(verifier.SELECTOR(), SEAL), IMAGE_ID, journalDigest);
        // Same statement, different bytes — still verifies.
        verifier.verify(
            abi.encodePacked(verifier.SELECTOR(), SEAL, bytes1(0x00)), IMAGE_ID, journalDigest
        );
    }

    /// Fuzz: every digest that is not the committed journal's fails pairing.
    function testFuzz_foreign_journal_digest_reverts(bytes32 digest) public {
        vm.assume(digest != sha256(JOURNAL));
        bytes memory fullSeal = abi.encodePacked(verifier.SELECTOR(), SEAL);
        vm.expectRevert();
        verifier.verify(fullSeal, IMAGE_ID, digest);
    }

    /// Documents exactly what the committed journal binds: three
    /// little-endian u64 — receipt_id = 7, owner = 0xA11CE, amount = 100.
    function test_journal_layout_decodes() public pure {
        assertEq(JOURNAL.length, 24);
        assertEq(_u64le(JOURNAL, 0), 7, "receipt_id");
        assertEq(_u64le(JOURNAL, 8), 0xA11CE, "owner");
        assertEq(_u64le(JOURNAL, 16), 100, "amount");
    }

    function _u64le(bytes memory data, uint256 offset) internal pure returns (uint64 v) {
        for (uint256 i = 0; i < 8; i++) {
            v |= uint64(uint8(data[offset + i])) << uint64(8 * i);
        }
    }
}
