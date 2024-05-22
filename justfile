default: build test

# Builds a debug build
build type=(""):
    cargo build {{type}}

# Runs Elvi on `./test.elv`
test: build
    cargo test
    cargo run -q -- ./test.elv

release:
    just build --release
