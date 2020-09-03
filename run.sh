cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/lines.wasm lines.wasm
wasm-strip lines.wasm
# cargo build --target wasm32-unknown-unknown
# cp target/wasm32-unknown-unknown/debug/hex_stat.wasm hex_stat.wasm%     