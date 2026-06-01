# Royal 3D Chess RS

[![Live Demo](https://img.shields.io/badge/Live-Demo-gold?style=for-the-badge)](https://aarambhdevhub.github.io/grandmaster-3d-chess-rs/)
[![GitHub Repository](https://img.shields.io/badge/GitHub-Repository-black?style=for-the-badge&logo=github)](https://github.com/AarambhDevHub/grandmaster-3d-chess-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/WebAssembly-WASM-654ff0?style=for-the-badge&logo=webassembly)](https://webassembly.org/)

**Royal 3D Chess RS** is a frontend-only Rust/WASM 3D chess game built with **Leptos**, **Scenix**, **Animato**, and the Rust `chess` crate.

Current version: **1.0.0**

## Live Demo

Play here:

**https://aarambhdevhub.github.io/grandmaster-3d-chess-rs/**

## Repository

GitHub:

**https://github.com/AarambhDevHub/grandmaster-3d-chess-rs**

## About

Royal 3D Chess RS is a browser-based 3D chess experience that runs completely on the client side.

There is no backend, WebSocket server, cloud authentication, Ollama service, or external API dependency. Everything runs locally in the browser through Rust compiled to WebAssembly.

This project is a showcase for:

- Rust frontend development
- WebAssembly games
- Leptos CSR applications
- Scenix-powered 3D rendering
- Animato-powered UI and scene animation
- Local chess logic using the Rust `chess` crate

## Features

- Full client-side Rust app compiled to WASM with Trunk.
- Frontend-only architecture with no backend dependency.
- Leptos CSR UI with main menu, setup screen, gameplay HUD, customization, settings, promotion flow, replay, and victory screen.
- 3D chessboard with procedural pieces, lighting, themed environments, weather, highlights, legal move markers, and capture effects.
- Mouse camera controls with drag-to-orbit and wheel-to-zoom.
- Legal move generation, FEN state, validation, captures, promotion, check, checkmate, stalemate, undo, and replay through `chess = "3.2.0"`.
- Local Rust bot with Easy, Medium, and Hard difficulty levels.
- Local browser persistence for settings, profile, customization, match history, and progress.
- Tailwind CSS v4 support through Trunk tooling.
- GitHub Pages deployment workflow.

## Tech Stack

- **Rust 2024**
- **Leptos `0.8.19`**
- **Scenix `1.1.0`**
- **Animato `1.4.0`**
- **Trunk**
- **Tailwind CSS `4.0.17`**
- **chess `3.2.0`**
- **WebGL/WASM**

## Project Type

This is a **frontend-only** chess game.

The original backend-style features are handled locally:

| Feature | Implementation |
|---|---|
| Bot play | Local Rust bot logic |
| Profile | Browser `localStorage` |
| Settings | Browser `localStorage` |
| Match history | Browser `localStorage` |
| Progress | Browser `localStorage` |
| Online-style room flow | Local pass-and-play style flow |
| Chess rules | Rust `chess` crate |
| Rendering | Scenix/WebGL |
| Animation | Animato |

## Prerequisites

Install Rust:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install the WASM target:

```sh
rustup target add wasm32-unknown-unknown
```

Install Trunk:

```sh
cargo install trunk
```

## Run Locally

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

For release build:

```sh
env NO_COLOR=false trunk build --release
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

Push to `main`, then enable GitHub Pages:

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

| Action | Control |
|---|---|
| Select piece | Left click |
| Move piece | Click a highlighted/legal target square |
| Rotate camera | Drag on the board |
| Zoom camera | Mouse wheel |
| Orbit view | `O` HUD button |
| Top view | `T` HUD button |
| Undo | `U` HUD button |
| Reset match | `R` HUD button |
| Settings | `S` HUD button or Settings on the main menu |

## Bot Difficulty

| Difficulty | Behavior |
|---|---|
| Easy | Relaxed legal move selection with light capture preference |
| Medium | Shallow search with human-like imperfect move choice |
| Hard | Stronger search, but still intentionally avoids engine-perfect play so games remain playable |

## Local Data

The app stores browser-local data in `localStorage`:

- Settings
- Profile identity
- Piece skins
- Emotes
- Match history
- Local progress

Clearing browser site data resets the app.

## Development Notes

This project is designed as a Rust/WASM frontend showcase. The goal is to keep the game fully playable in the browser without requiring any server setup.

Recommended checks before pushing:

```sh
cargo fmt
cargo test
cargo check --target wasm32-unknown-unknown
env NO_COLOR=false trunk build
```

## License

This project is licensed under the **MIT License**.

See the [LICENSE](LICENSE) file for details.

## Author

Built by **Aarambh Dev Hub**.

GitHub: https://github.com/AarambhDevHub