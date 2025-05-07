---
sidebar_position: 1
title: For Developers
---

If you want to build the application from source or contribute to development, follow these steps:

## Technologies Used

- **[Svelte](https://svelte.dev/)**: Frontend framework for the user interface.
- **[Rust](https://www.rust-lang.org/)**: Backend language for core logic and Tauri integration.
- **[Tauri](https://tauri.app/)**: Framework for building cross-platform desktop applications with web frontends.
- **[MongoDB](https://www.mongodb.com/)**: Database for storing package and stage configurations.

## Prerequisites

- **Node.js and npm**: For managing frontend dependencies and running scripts. (Download from [nodejs.org](https://nodejs.org/))
- **Rust and Cargo**: For building the backend. (Install via [rustup.rs](https://rustup.rs/))
- **MongoDB**: (Optional, can be configured in-app) A running MongoDB instance if you want to persist configurations.

## Steps

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/ClementHector/RezLauncher.git
    cd RezLauncher
    ```

2.  **Install frontend dependencies:**
    ```bash
    npm install
    ```

3.  **Build and run the application:**

    *   For development (with hot-reloading):
        ```bash
        npm run tauri dev
        ```
    *   For a production build:
        ```bash
        npm run tauri build
        ```
        The executable will be located in `src-tauri/target/release/`.
