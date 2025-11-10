# Dunfog

Dunfog is an old-school style roguelike with turn based combat, item crafting, procedurally generated floors, and a boss at the end!

Made for hackclub's siege! entire project took nearly 30 hours (in one week!)

## Controls!

click on a tile to move there, click on an enemy (within range of your weapon) to attack it.

​use F to open Inventory. use E (when prompted) to interact with a tile.


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