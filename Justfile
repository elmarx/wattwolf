host := "gary.dyn.athmer.org"

build:
    cross build --target=aarch64-unknown-linux-gnu

scp: build
    scp ./target/aarch64-unknown-linux-gnu/debug/wattwolf elmar@{{ host }}:
