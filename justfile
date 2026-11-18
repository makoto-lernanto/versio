di: formo clippy provo sentineleto arm64 arm64-provo arm64-integro kani dokumentaro
formo:
    cargo fmt --all -- --check
    cargo fmt --manifest-path arm64_provo/Cargo.toml -- --check
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo clippy --workspace --all-targets --no-default-features -- -D warnings
    cd arm64_provo && RUSTFLAGS="-C link-arg=-Tlinker.ld" cargo clippy --release --target aarch64-unknown-none -- -D warnings
provo:
    cargo test --workspace --all-features
sentineleto:
    cargo build -p no_std_provo --target x86_64-unknown-none
    cargo build -p no_std_provo --target aarch64-unknown-none
    cargo clippy -p libversio --no-default-features --target aarch64-unknown-none -- -D warnings
arm64:
    RUSTFLAGS="-C linker=$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin/rust-lld" cargo build --release -p versio-ki --target aarch64-unknown-linux-musl
arm64-provo:
    cd arm64_provo && RUSTFLAGS="-C link-arg=-Tlinker.ld" cargo build --release --target aarch64-unknown-none
    qemu-system-aarch64 -M virt -cpu cortex-a53 -m 256M -nographic -semihosting-config enable=on,target=native -kernel arm64_provo/target/aarch64-unknown-none/release/arm64_provo
arm64-integro: arm64
    rm -rf target/arm64-integro
    mkdir -p target/arm64-integro
    cd target/arm64-integro && qemu-aarch64 {{justfile_directory()}}/target/aarch64-unknown-linux-musl/release/vs ini
    printf 'saluton arm64\n' > target/arm64-integro/testo.txt
    cd target/arm64-integro && qemu-aarch64 {{justfile_directory()}}/target/aarch64-unknown-linux-musl/release/vs komitu --noto "arm64"
    test -f target/arm64-integro/.vs/KAPO
    test "$(ls target/arm64-integro/.vs/objektoj | wc -l)" -eq 3
# Nur en GitHub Actions: ne rulu loke (por eviti memorelspezon).
kani:
    cargo kani -p libversio
dokumentaro:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
