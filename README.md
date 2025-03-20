# asciihou

```sh
cargo install wasm-bindgen-cli          
```

```sh
rustup target install wasm32-unknown-unknown 
```

```sh
 cargo build --release --target wasm32-unknown-unknown
```

```sh
wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "asciihou" ./target/wasm32-unknown-unknown/release/asciihou.wasm
```