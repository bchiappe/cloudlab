# Cloudlab Project Wiki

## Overview
This is a Server-Side Rendered (SSR) web application utilizing a modern Rust stack for both the backend logic and the frontend visual layer.

- **Frontend & Reactivity**: **Leptos**
- **Backend Server**: **Axum**
- **Async Runtime**: **Tokio**
- **Client Deployment Target**: **WebAssembly (`wasm32-unknown-unknown`)**

## Project Structure
The project was originally scaffolded using the `leptos-rs/start-axum` template. 
- `src/main.rs`: Entry point for the Axum server. It builds the HTTP router, integrates it with the Leptos SSR routes, and maps out the static file directories.
- `src/lib.rs`: Connects the main App logic and handles the WebAssembly client-side hydration flow.
- `src/app.rs`: Contains the root Leptos UI `#[component]` layout. This serves as the foundation for the visual interfaces.
- `Cargo.toml`: Managed with features for `ssr` and `hydrate` allowing dual-compilation behavior between server execution and client Wasm loading.

## Development Workflow
The app requires `cargo-leptos` to be run correctly and builds two separate targets concurrently (server-side binary and client-side WASM).

```bash
# Start the live-reloading development server
cargo leptos watch

# Build for release
cargo leptos build --release
```

## Essential Guidelines for LLMs
For AI coding assistants interpreting this codebase:
- **Rust First**: The entire web logic, spanning from server routes to client-side DOM interactivity, is managed in Rust using Leptos Signals and Contexts.
- **Dual Execution**: Keep in mind that component logic generally runs twice: once on the Axum server rendering HTML, and again on the browser where it hydrates WebAssembly interactions. Code using strictly browser or backend API requirements (e.g. `window`, `fs`) must be carefully gated or run within explicit `create_effect` hooks or server blocks.
- **Architecture**: Stick to component-driven design when modifying UI, maintaining strong types inside AppState when extending the backend.

*Note: Maintain and update this file as system architecture, design systems, or substantial new features are introduced.*
