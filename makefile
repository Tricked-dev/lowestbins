start:
	cargo build --release; ./target/release/lowestbins
test:
	cargo test --test test_data
bench:
	cargo bench
br:
	cargo +nightly build -Z build-std=std,panic_abort --target x86_64-unknown-linux-gnu --release
	