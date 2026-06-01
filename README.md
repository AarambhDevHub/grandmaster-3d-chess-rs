# Royal 3D Chess RS

Frontend-only Rust rebuild of `grandmaster-3d-chess` using Leptos, Scenix, Animato, and the Rust `chess` crate.

## Run

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```

Open the URL printed by Trunk, usually `http://127.0.0.1:8080`.

If your shell exports `NO_COLOR=1` and Trunk rejects it, run:

```sh
env -u NO_COLOR trunk serve
```

## Checks

```sh
cargo test
cargo check --target wasm32-unknown-unknown
trunk build
```

This app is frontend-only. Sign-in, profile, settings, campaign progress, and match history are local browser data. Bot moves are generated in Rust; no backend, WebSocket, Ollama, or API calls are used.
