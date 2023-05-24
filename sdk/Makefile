all: simulated sdk

simulated:
	@cargo build --target=x86_64-unknown-linux-gnu
	@cargo build --example simulated --target=x86_64-unknown-linux-gnu

sdk:
	@cargo build -r --target=aarch64-unknown-linux-gnu

sdk-example:
	@RUSTFLAGS="-C target-feature=+crt-static" cargo build -r --example aarch64 --target=aarch64-unknown-linux-gnu
	

test:
	@cargo test -- --nocapture

run_simulated:
	@cargo run --example simulated --target=x86_64-unknown-linux-gnu

fvp: sdk-example
	@install -m755 ../out/aarch64-unknown-linux-gnu/release/examples/aarch64 ../out/shared/sdk-example
