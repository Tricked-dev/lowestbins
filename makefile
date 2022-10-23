start:
	cargo build --release; ./target/release/lowestbins
test:
	cargo test --test test_data
bench:
	cargo bench
	