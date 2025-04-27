# rezLauncher

rezLauncher is a web-based application designed to manage and launch different stages of a production using rezStage. It provides a user-friendly interface to create, manage, and load stages, making it easier for users to work with different environments.

Two modes are available in the web UI: default and developer mode.

## Default Mode

The default mode is the standard mode of the web UI. It allows users to navigate through the different stages. In this mode, users can only view the stages stored in MongoDB. Users can select a stage, and the web UI will create the rez environment with the selected stage.

**Feature:**
- Load => Load the selected stage and create the rez environment.

## Developer Mode

The developer mode is the advanced mode of the web UI. This mode allows users to view and manage all stages stored in MongoDB.

**Features:**
- Bake => Bake a new stage and store it in MongoDB. A popup will appear to ask for the name of the new stage. If the name already exists, the previous version will be archived, and the new version will be stored with the provided name.
- Revert => Revert a stage to a previous version stored in MongoDB. A popup will appear to select the version to revert to. The current version will be replaced by the selected version.
- Edit => Edit the details of a stage directly in MongoDB.

### UI Design

#### Layout in Default Mode

┌──────────────────────────────────────────────────────────────────────────────┐
│ [RezLauncher Logo]                         Mode: [🔘 Default] [⚪ Developer]│
│                                                                    [🔘 Dark]│
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ [ project ▾] / [ modeling ▾ ] / [ maya ▾ ] / ...                             │
├─────────────────────────────────────┬────────────────────────────────────────┤
│                                         │                                    │
│ ▸ Available Stages                      │               Alias                │
│ ├─ dev                 [Load]           │                                    │
│ ├─ beta                [Load]           │     ▸ Maya                [Load]   │
│                                         │                                    │
│                                         │     ▸ Rv                  [Load]   │
│                                         │                                    │
│                                         │                                    │
│                                         │                                    │
│                                         │                                    │
│                                         │                                    │
│                                         │                                    │
│                                         │                                    │
│                                         │                                    │
├─────────────────────────────────────────┴────────────────────────────────────┤
│ Logs / Output :                                                              │
│                                                                              │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘

#### Layout in Developer Mode

┌──────────────────────────────────────────────────────────────────────────────┐
│ [RezLauncher Logo]                         Mode: [🔘 Default] [⚪ Developer]│
│                                                                    [🔘 Dark]│
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ [ project ▾] / [ modeling ▾ ] / [ maya ▾ ] / ...                             │
├─────────────────────────────────────────┬────────────────────────────────────┤
│                                         │                                    │
│ ▸ Available Stages                      │                Alias               │
│ ├─ dev                 [Load] [Revert]  │                                    │
│ ├─ beta                [Load] [Revert]  │      ▸ Maya             [Load]     │
│                                         │                                    │
│ ▸ Available package collection          │      ▸ Rv               [Load]    │
│ ├─ 0.1.2               [Bake] [Edit]    │                                    │
│ ├─ ticket#123          [Bake] [Edit]    │                                    │
│ + Create new package collection         │                                   │
│                                         │                                    │
│                                         │                                    │
├─────────────────────────────────────────┴────────────────────────────────────┤
│ Logs / Output :                                                              │
│ [✔] Baked 0.1.2 into dev                                                     │
│ [⚠] Package modelingTools not found                                         │
└──────────────────────────────────────────────────────────────────────────────┘