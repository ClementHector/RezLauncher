---
sidebar_position: 3
title: Getting Started
---

## Usage

1.  **Initial Setup**:
    *   On the first launch, if a MongoDB connection URI is not found in your settings, a configuration modal will appear.
    *   Enter your MongoDB connection string (e.g., `mongodb://localhost:27017`) and test the connection. The application will store this URI for future sessions.

2.  **Navigating Layers**:
    *   Use the "LAYERS" breadcrumb navigation at the top to define your working context.
    *   Select or add new options for Project, Modeling Type, and Application. This creates a URI path (e.g., `/MyProject/Character/Maya`) that filters the displayed package collections and stages.

3.  **Modes**:
    *   **Config Mode**:
        *   **Package Collections Panel**:
            *   Click "Create New" to define a new package collection for the current URI. Specify version, packages (e.g., `maya-2023`, `houdini-19.5`), inheritance, and tools.
            *   Click the "Bake" icon on a package collection to create a "Stage" from it. This typically involves generating an RXT file and saving it as a new, active stage.
            *   Click the "Edit" icon to create a new package collection based on an existing one.
        *   **Stages Panel**:
            *   View stages created for the current URI.
            *   Click the "Revert" icon to make an older version of a stage active.
    *   **Launcher Mode** (and generally available actions):
        *   **Package Collections / Stages Panels**:
            *   Click on a package collection or a stage to select it. This will populate the "Tools" panel.
            *   Click the "Load" (play) icon to open a Rez environment for the selected item in a new terminal.
        *   **Tools Panel**:
            *   View tools associated with the selected package collection or stage.
            *   Click the "Load" (play) icon next to a tool to launch that specific tool within the context of its parent collection/stage in a new terminal.

4.  **Interface**:
    *   **Home Button**: Resets the URI path to the root.
    *   **Refresh Button**: Reloads package collections and stages for the current URI.
    *   **Theme Toggle**: Switch between Light and Dark themes.
    *   **Logs Panel**: View real-time logs of application actions, successes, warnings, and errors at the bottom of the window.
