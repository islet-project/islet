all: simulated sdk

simulated:
	@cargo build --target=x86_64-unknown-linux-gnu
	@cargo build --example simulated --target=x86_64-unknown-linux-gnu

sdk:
	@cargo build -Zbuild-std=panic_abort,std --target=aarch64-unknown-linux-gnu
	@cargo build --example arm64 -Zbuild-std=panic_abort,std --target=aarch64-unknown-linux-gnu

test:
	@cargo test -- --nocapture

run_simulated:
	@cargo run --example simulated --target=x86_64-unknown-linux-gnu
