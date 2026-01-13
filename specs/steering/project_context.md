# Project Context

## Directory Structure

```
├── .cursor/rules/       # Steering documents (Context)
├── specs/               # Product Requirements & Design Specs
│   ├── requirement/     # PRDs
│   ├── design/          # Technical Design Docs
│   └── task/            # Task Tracking Lists
├── src/                 # Frontend (Vue.js)
│   ├── assets/          # CSS, Images, Icons
│   ├── components/      # Vue Components
│   ├── composables/     # Shared Logic Hooks
│   ├── types/           # TS Interfaces
│   └── __tests__/       # Frontend Tests
├── src-tauri/           # Backend (Rust)
│   ├── src/
│   │   ├── commands/    # Tauri Command Handlers
│   │   ├── core/        # Shared Types, Config, Events
│   │   ├── services/    # Business Logic
│   │   ├── system/      # OS Integration (Tray, Hotkeys)
│   │   ├── integrations/# External API Clients
│   │   └── security/    # Auth & Encryption
│   └── tests/           # Integration Tests
└── README.md            # Quickstart Guide
```

## Key Files
-   `src-tauri/src/lib.rs`: Rust library entry point, exports modules and state initialization.
-   `src-tauri/src/main.rs`: Rust binary entry point, runs the Tauri app.
-   `src/App.vue`: Root frontend component, handles layout and global overlays (Flight Console).
-   `src/composables/useTauri.ts`: Typed bridge between Frontend and Backend.
-   `src/assets/design-system.css`: Core theme definitions.

## Workflow Status
-   **Current Phase**: Phase 10 (Deployment & Distribution).
-   **Completed**: Core Backend, Frontend Foundation, Background Services, Unit/Integration Tests.
-   **Next**: Packaging, CI/CD setup, Release.
