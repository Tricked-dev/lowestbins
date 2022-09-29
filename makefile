start:
	cargo build --release; ./target/release/lowestbins
buildb:
	cd bench-lb && cargo build --release

	