# Show help by default
default:
	@just --list --justfile {{ justfile() }}

# Builds and runs a development binary
run *extra_args:
	RUST_BACKTRACE=1 cargo run -- {{ extra_args }}

# Builds a development binary (target/debug/*)
build-dev *extra_args:
	RUST_BACKTRACE=1 cargo build {{ extra_args }}

# Builds an optimized release binary (target/release/*)
build-release *extra_args: (build-dev "--release")

# Runs the clippy linter
clippy *extra_args:
	cargo clippy {{ extra_args }}

# Runs tests
test:
	cargo test
