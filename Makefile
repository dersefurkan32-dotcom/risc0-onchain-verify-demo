.PHONY: test-onchain fmt-onchain help-guest

test-onchain:
	cd onchain-verify && forge test -vv

fmt-onchain:
	cd onchain-verify && forge fmt

help-guest:
	cd identity-bind && RISC0_DEV_MODE=1 cargo run --release -- --help
