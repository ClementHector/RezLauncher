# rezLauncher

A modern desktop application for managing Rez packages, stages, and tools in production pipelines. Built with Tauri, Svelte, and MongoDB.

## Overview

rezLauncher is a desktop application designed to streamline workflow management in production pipelines. It provides an intuitive interface for managing package collections, stages, and tools across different projects.

## Key Features

- **Package Collection Management**: Create, view, and manage versioned package collections
- **Stage Management**: Create stages from package collections and track their history
- **Tool Loading**: Launch tools associated with package collections or stages
- **URI-based Navigation**: Navigate through projects, modeling types, and applications with a hierarchical URI system
- **Modes**: Switch between Launcher and Config modes with different feature sets
- **Dark/Light Theme**: Toggle between dark and light themes for comfortable viewing in any environment
- **Logging System**: Built-in logging to track operations and system status

## Prerequisites

- [MongoDB](https://www.mongodb.com/try/download/community) (running on localhost:27017)
- [Node.js](https://nodejs.org/) (v16 or higher)
- [Rust](https://www.rust-lang.org/tools/install) (for Tauri backend)

## Installation

1. Clone the repository:
   ```
   git clone <repository-url>
   cd rezLauncher
   ```

2. Install dependencies:
   ```
   npm install
   ```

3. Ensure MongoDB is running:
   ```
   mongod --dbpath <your-data-path>
   ```

4. Build the application:
   ```
   npm run tauri build
   ```

5. The built application will be available in the `src-tauri/target/release` directory.

## Development

For development, you can run the application in development mode:

```
npm run tauri dev
```

This will start the Svelte dev server and launch the Tauri application window.

## Project Structure

- `src/`: Svelte frontend application
  - `lib/`: Components and type definitions
  - `routes/`: Svelte routes
- `src-tauri/`: Rust backend and Tauri configuration
  - `src/`: Rust source code
  - `icons/`: Application icons

## Data Models

### Package Collection

A collection of packages with associated tools and version information:
- Version
- Packages list
- Tools list
- URI path
- Creation metadata

### Stage

A snapshot of a package collection at a specific point:
- Name
- URI
- From version
- Rez template path
- Tools list
- Active status

## Configuration

The MongoDB connection is configured in `src-tauri/src/main.rs`. By default, it connects to:
- MongoDB URI: `mongodb://localhost:27017`
- Database name: `rez_launcher`

## Backend Testing (Rust)

Unit tests for the Rust backend (Tauri core process) are located within the `src-tauri/src/main.rs` file, inside the `#[cfg(test)] mod tests` module.

To run the tests:

1.  **Ensure a MongoDB instance is running locally** on the default port (27017). The tests require a running MongoDB server to connect to the test database.
2.  Navigate to the `src-tauri` directory in your terminal:
    ```bash
    cd src-tauri
    ```
3.  Run the tests using Cargo:
    ```bash
    cargo test
    ```

The tests will connect to MongoDB, create a temporary test database (`rez_launcher_test_db`), run the test cases, and then drop the test database.

## License

<!-- TODO -->

## Credits

Built with:
- [Tauri](https://tauri.app/) - Desktop application framework
- [Svelte](https://svelte.dev/) - Frontend framework
- [MongoDB](https://www.mongodb.com/) - Database
- [Rust](https://www.rust-lang.org/) - Backend language