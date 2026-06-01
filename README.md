# Royal 3D Chess RS

Royal 3D Chess RS is a frontend-only Rust/WASM 3D chess game built with Leptos, Scenix, Animato, and the Rust `chess` crate.

Current version: **1.0**

GitHub: https://github.com/AarambhDevHub/grandmaster-3d-chess-rs

This project is a browser showcase for:

- [Leptos](https://leptos.dev/)
- Scenix
- Animato
- Rust/WASM 3D frontend development

It runs entirely in the browser. There is no backend, WebSocket server, cloud auth, Ollama service, or API dependency.

## Features

- Full client-side Rust app compiled to WASM with Trunk.
- Leptos CSR UI with main menu, setup, gameplay HUD, customization, settings, promotion, replay, and victory flow.
- 3D chessboard with procedural pieces, lighting, themed environments, weather, highlights, move markers, and capture effects.
- Mouse camera controls: drag to orbit and wheel to zoom.
- Legal move generation, FEN state, validation, captures, promotion, check, checkmate, stalemate, undo, and replay through `chess = "3.2.0"`.
- Local Rust bot with Easy, Medium, and Hard personalities.
- Local browser persistence for settings, profile, customization, match history, and progress.
- Tailwind CSS v4 via native Trunk tool configuration.

## Tech Stack

- Rust 2024
- Leptos `0.8.19`
- Scenix `1.1.0`
- Animato `1.4.0`
- Trunk
- Tailwind CSS `4.0.17`
- `chess = "3.2.0"`
- WebGL/WASM

## Prerequisites

Install Rust:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
````

Install the WASM target:

```sh
rustup target add wasm32-unknown-unknown
```

Install Trunk:

```sh
cargo install trunk
```

## Run

From the project root:

```sh
env NO_COLOR=false trunk serve
```

Open the URL printed by Trunk, usually:

```text
http://127.0.0.1:8080
```

`NO_COLOR=false` avoids a Trunk CLI parsing issue on shells that export `NO_COLOR=1`.

## Build

```sh
env NO_COLOR=false trunk build
```

## Test

```sh
cargo test
cargo check --target wasm32-unknown-unknown
env NO_COLOR=false trunk build
```

## Deploy to GitHub Pages

This repository includes:

```text
.github/workflows/pages.yml
```

Push to `main`, then enable GitHub Pages with:

```text
Settings → Pages → Source: GitHub Actions
```

The workflow builds with:

```sh
env NO_COLOR=false trunk build --release --public-url "/grandmaster-3d-chess-rs/"
```

After deployment, the app will be available at:

```text
https://aarambhdevhub.github.io/grandmaster-3d-chess-rs/
```

For a custom domain, update the `--public-url` value in the workflow.

## Controls

* Select piece: left click
* Move piece: click a highlighted/legal target square
* Rotate camera: drag on the board
* Zoom camera: mouse wheel
* Orbit view: `O` HUD button
* Top view: `T` HUD button
* Undo: `U` HUD button
* Reset match: `R` HUD button
* Settings: `S` HUD button or Settings on the main menu

## Bot Difficulty

* Easy: relaxed legal move selection with light capture preference.
* Medium: shallow search with human-like imperfect move choice.
* Hard: stronger search, but still intentionally avoids engine-perfect play so games remain playable.

## Local Data

The app stores browser-local data in `localStorage`:

* Settings
* Profile identity
* Piece skins and emotes
* Match history
* Local progress

Clearing browser site data resets the app.

## Repository

```text
https://github.com/AarambhDevHub/grandmaster-3d-chess-rs
```

## License

This project is licensed under the MIT License.

See the [LICENSE](LICENSE) file for details.
