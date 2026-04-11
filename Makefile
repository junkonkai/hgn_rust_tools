.PHONY: install-dev

install-dev:
	cargo build -p ogp-generator --release
	cp target/release/ogp-generator /usr/local/bin/hgn/ogp-generator
