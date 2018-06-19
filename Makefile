all: docs

test:
	cargo test
	cargo test -- --ignored

docs: docs/snail.7 docs/snaild.8 docs/snailctl.8

docs/%: docs/%.scd
	scdoc < $^ > $@
