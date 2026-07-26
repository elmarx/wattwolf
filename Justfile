host := "gary.dyn.athmer.org"

ci:
    cargo check --all-targets
    cargo fmt --check
    # deny default clippy violations
    cargo clippy --all-targets -- -D warnings
    cargo clippy --all-targets -- -W clippy::pedantic
    cargo nextest run

build_nix:
    nix build .#default

build:
    cross build --target=aarch64-unknown-linux-gnu

scp: build
    scp ./target/aarch64-unknown-linux-gnu/debug/wattwolf elmar@{{ host }}:
