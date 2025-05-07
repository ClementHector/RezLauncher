---
sidebar_position: 1
---

# Introduction to RezLauncher

RezLauncher is an application designed to simplify and streamline your workflows when working with Rez packages. This documentation will guide you through installation, usage, and development of RezLauncher.


RezLauncher is a modern desktop application designed to simplify the management and launching of software environments powered by the Rez package manager. It provides a user-friendly interface to configure, version, and deploy collections of software packages (stages) and their associated tools.

## Key Features

- **MongoDB Integration**: Connect to a MongoDB instance to store and retrieve package and stage configurations.
- **Dynamic URI Navigation**: Organize and access configurations using a hierarchical URI structure (e.g., `/Project/ModelingType/Application`).
- **Package Collection Management**:
    - Create and version package collections.
    - Define packages, inheritance, and associated tools for each collection.
- **Stage Management**:
    - "Bake" package collections into immutable stages.
    - View stage history and details.
    - Revert to previous stage versions.
    - Activate/deactivate stages.
- **Tool Launching**:
    - Launch Rez environments for selected package collections or stages directly into a terminal.
    - Launch individual tools associated with a collection or stage.
- **Configuration Modes**:
    - **Config Mode**: For setting up and managing package collections and baking stages.
    - **Launcher Mode**: For browsing and launching existing stages and tools.
- **User Interface**:
    - Intuitive Svelte-based frontend.
    - Light and Dark theme options.
    - Real-time logging panel for monitoring application activity.
- **Cross-Platform**: Built with Tauri for desktop compatibility.
