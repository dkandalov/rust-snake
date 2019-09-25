cargo = /usr/local/Cellar/rust/1.37.0_1/bin/cargo

build-and-run:
	$(cargo) run --color=always --package snake --bin snake

test:
	$(cargo) test

release:
	$(cargo) build --release && strip target/release/snake

valgrind-massif:
	valgrind --tool=massif --massif-out-file=massif.out --time-unit=B ./target/debug/snake && ms_print massif.out > massif.out.printed
