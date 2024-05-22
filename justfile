default: build test

# Builds a debug build
build type=(""):
    cargo build

# Runs Elvi on `./test.elv`
test: build
    cargo run -q -- ./test.elv

release:
    just build --release
