# dunfog

dunfog is an old-school style roguelike with turn based combat, item crafting, and a boss at the end!

made for hackclub's siege! entire project took nearly 30 hours (in one week!)

## Building from source

this project is made in rust so obviously you'll need rust (with cargo) installed.

to run standalone you can just do:
```bash
cargo run
```

and to build for web and host on localhost with `basic-http-server`, do 
```bash
cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/dunfog.wasm web/ && basic-http-server web/
```