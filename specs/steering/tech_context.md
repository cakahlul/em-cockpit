# Technology Context

## Tech Stack
-   **Frontend**: Vue.js 3, TypeScript, Vite.
-   **Backend**: Rust (Tauri v2).
-   **State Management**:
    -   Frontend: Vue Composition API (`ref`, `computed`).
    -   Backend: Rust `Arc<RwLock<AppState>>`.
-   **Styling**: Vanilla CSS with CSS Variables, implementing a "Glassmorphism" design system.
-   **Persistence**: SQLite (via `rusqlite`) for caching, `keyring` crate for secure credential storage.

## Architecture Patterns
-   **Local-First Service**: Logic lives in Rust services (`SearchService`, `PrAggregator`).
-   **Event-Driven**: A central `EventBus` (Observer pattern) in Rust broadcasts state changes (`TrayStateChanged`, `PrDataUpdated`) to subscribers (System Tray, Frontend).
-   **Background Polling**: Independent `BackgroundPoller` service fetches external data periodically.
-   **IPC**: Frontend invokes Rust commands via `invoke()`; Rust pushes events via Tauri events.

## Development Conventions
-   **TDD**: Rust backend development follows Test-Driven Development. Unit tests are co-located in modules (`#[cfg(test)]`), integration tests in `tests/`.
-   **Error Handling**: Typed errors using `thiserror`. Errors propagate from Service -> Command -> Frontend.
-   **Styling**: No Tailwind. Use semantic CSS variables defined in `src/assets/design-system.css`.
-   **Components**: Vue components are Single File Components (SFC) using `<script setup lang="ts">`.

## Constraints
-   **OS**: macOS primarily (key shortcuts and tray behavior optimized for macOS).
-   **Offline Capable**: App should remain functional (showing cached data) when offline.
