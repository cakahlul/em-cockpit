# The EM Cockpit – Implementation Tasks (TDD)

## Overview

This document provides a comprehensive task breakdown for implementing The EM Cockpit using **Test-Driven Development (TDD)**. All tasks follow the RED-GREEN-REFACTOR-VERIFY cycle.

**TDD Workflow:**
1. 🔴 **RED**: Write failing tests first
2. 🟢 **GREEN**: Write minimal code to pass tests
3. 🔵 **REFACTOR**: Improve code quality while keeping tests green
4. ✅ **VERIFY**: Run full test suite and validate

---

## Phase 1: Foundation & Infrastructure

### Task 1.1: Project Setup & Build Configuration

**Description:** Initialize Tauri project with Rust backend and Vue.js frontend

**TDD Requirements:**
- **Test Strategy**: Integration tests for build pipeline, unit tests for config loading
- **Success Criteria**: `npm run tauri dev` starts app, all config loads correctly
- **Test Coverage Target**: 90% for config modules
- **Key Test Cases**:
  - App launches without errors
  - Config files parse correctly
  - Environment variables load properly
- **Verification Method**: Automated CI build + manual app launch

**Implementation Steps:**
1. 🔴 Write test for config loading
2. 🟢 Initialize Tauri project with Vue.js
3. 🔵 Refactor build scripts for optimization
4. ✅ Verify CI pipeline passes

---

### Task 1.2: Security Layer - Credential Manager

**Description:** Implement secure credential storage using OS keychain

**TDD Requirements:**
- **Test Strategy**: Unit tests with mock keychain, integration tests with OS keychain
- **Success Criteria**: Credentials stored/retrieved securely, panic wipe clears all data
- **Test Coverage Target**: 100% (security-critical)
- **Key Test Cases**:
  - Store credential successfully
  - Retrieve credential matches stored value
  - Delete credential removes from keychain
  - Panic wipe clears all credentials
  - Handle keychain access errors gracefully
- **Verification Method**: Automated tests + manual keychain inspection

**Implementation Steps:**
1. 🔴 Write failing tests for `CredentialManager::store_credential()`
2. 🟢 Implement credential storage using `keyring` crate
3. 🔴 Write tests for `retrieve_credential()` and `delete_credential()`
4. 🟢 Implement retrieval and deletion
5. 🔴 Write tests for `panic_wipe()`
6. 🟢 Implement panic wipe functionality
7. 🔵 Refactor error handling
8. ✅ Verify all tests pass, manual keychain check

**Files:**
- `src/security/credential_manager.rs`
- `src/security/credential_manager_tests.rs`

---

### Task 1.3: Cache Service - Multi-Tier Caching

**Description:** Implement three-tier caching (memory, SQLite, network)

**TDD Requirements:**
- **Test Strategy**: Unit tests for each cache tier, integration tests for fallback behavior
- **Success Criteria**: Cache hits/misses work correctly, TTL expiration functions, tier promotion works
- **Test Coverage Target**: 95%
- **Key Test Cases**:
  - Memory cache stores and retrieves values
  - DB cache persists across restarts
  - Expired entries not returned
  - Cache promotion from DB to memory works
  - Concurrent access is thread-safe
- **Verification Method**: Automated tests + performance benchmarks

**Implementation Steps:**
1. 🔴 Write tests for memory cache operations
2. 🟢 Implement in-memory LRU cache
3. 🔴 Write tests for SQLite cache persistence
4. 🟢 Implement DB cache layer
5. 🔴 Write tests for TTL expiration
6. 🟢 Implement expiration logic
7. 🔴 Write tests for tier promotion
8. 🟢 Implement cache promotion
9. 🔵 Refactor for thread safety
10. ✅ Run benchmarks, verify all tests pass

**Files:**
- `src/services/cache_service.rs`
- `src/services/cache_service_tests.rs`

---

## Phase 2: System Integration Layer

### Task 2.1: Global Hotkey Management

**Description:** Implement cross-platform global hotkey listener

**TDD Requirements:**
- **Test Strategy**: Unit tests for hotkey parsing, integration tests for registration
- **Success Criteria**: Hotkey triggers app window, configurable shortcuts work
- **Test Coverage Target**: 85%
- **Key Test Cases**:
  - Parse hotkey string correctly (e.g., "Alt+Space")
  - Register hotkey with OS
  - Hotkey event triggers callback
  - Unregister hotkey on app close
  - Handle conflicting hotkeys gracefully
- **Verification Method**: Automated tests + manual OS-level testing

**Implementation Steps:**
1. 🔴 Write tests for hotkey string parsing
2. 🟢 Implement parser for shortcut notation
3. 🔴 Write tests for hotkey registration
4. 🟢 Integrate `tauri-plugin-global-shortcut`
5. 🔴 Write tests for event callbacks
6. 🟢 Implement event handler system
7. 🔵 Refactor for cross-platform compatibility
8. ✅ Test on macOS, Windows, Linux

**Files:**
- `src/system/hotkey.rs`
- `src/system/hotkey_tests.rs`

---

### Task 2.2: System Tray (Tray Beacon)

**Description:** Implement system tray with state-based icon and context menu

**TDD Requirements:**
- **Test Strategy**: Unit tests for state transitions, integration tests for tray updates
- **Success Criteria**: Tray icon reflects app state (Green/Amber/Red), menu actions work
- **Test Coverage Target**: 90%
- **Key Test Cases**:
  - Tray icon initializes correctly
  - State changes update icon color
  - Tooltip shows current status
  - Click opens Flight Console
  - Context menu items trigger correct actions
  - Icon transitions animate smoothly
- **Verification Method**: Automated tests + visual inspection

**Implementation Steps:**
1. 🔴 Write tests for `TrayState` enum and transitions
2. 🟢 Implement state management
3. 🔴 Write tests for icon updates
4. 🟢 Implement tray icon rendering
5. 🔴 Write tests for context menu
6. 🟢 Implement menu actions
7. 🔵 Refactor for animation smoothness
8. ✅ Visual testing on all platforms

**Files:**
- `src/system/tray.rs`
- `src/system/tray_tests.rs`

---

## Phase 3: Integration Layer - External APIs

### Task 3.1: Jira Integration - Repository Pattern

**Description:** Implement Jira ticket repository with search and detail fetching

**TDD Requirements:**
- **Test Strategy**: Unit tests with mocked HTTP client, integration tests with Jira sandbox
- **Success Criteria**: Fetch ticket by ID, search tickets with JQL, map Jira responses to domain models
- **Test Coverage Target**: 95%
- **Key Test Cases**:
  - `find_by_id()` returns correct ticket
  - `search()` with JQL returns filtered results
  - API errors handled gracefully (401, 404, 500)
  - Rate limiting respected
  - Response mapping preserves all fields
- **Verification Method**: Unit tests with mock responses + sandbox integration tests

**Implementation Steps:**
1. 🔴 Write tests for `TicketRepository` trait
2. 🟢 Define trait interface
3. 🔴 Write tests for `JiraClient::find_by_id()` with mock HTTP
4. 🟢 Implement ticket fetching
5. 🔴 Write tests for `search()` with JQL
6. 🟢 Implement search with JQL builder
7. 🔴 Write tests for error scenarios
8. 🟢 Implement error handling
9. 🔵 Refactor HTTP client and mapper
10. ✅ Run integration tests against Jira sandbox

**Files:**
- `src/integrations/traits.rs`
- `src/integrations/jira/client.rs`
- `src/integrations/jira/models.rs`
- `src/integrations/jira/mapper.rs`
- `src/integrations/jira/client_tests.rs`

---

### Task 3.2: Git Hosting Integration - Strategy Pattern

**Description:** Implement Git PR repository for Bitbucket, GitHub, GitLab

**TDD Requirements:**
- **Test Strategy**: Unit tests per provider with mocks, integration tests with test repositories
- **Success Criteria**: Fetch PRs by reviewer, detect stale PRs, support all three providers
- **Test Coverage Target**: 95%
- **Key Test Cases**:
  - `find_by_reviewer()` filters correctly
  - Stale detection calculates duration properly
  - Provider-specific API differences handled
  - OAuth token refresh works
  - Pagination handles large PR lists
- **Verification Method**: Unit tests + integration tests with test repos

**Implementation Steps:**
1. 🔴 Write tests for `PullRequestRepository` trait
2. 🟢 Define trait interface
3. 🔴 Write tests for Bitbucket client
4. 🟢 Implement `BitbucketClient`
5. 🔴 Write tests for GitHub client
6. 🟢 Implement `GitHubClient`
7. 🔴 Write tests for GitLab client
8. 🟢 Implement `GitLabClient`
9. 🔴 Write tests for provider factory
10. 🟢 Implement `GitProvider` enum
11. 🔵 Refactor common HTTP logic
12. ✅ Integration tests with all three providers

**Files:**
- `src/integrations/git/mod.rs`
- `src/integrations/git/bitbucket.rs`
- `src/integrations/git/github.rs`
- `src/integrations/git/gitlab.rs`
- `src/integrations/git/models.rs`
- `src/integrations/git/*_tests.rs`

---

### Task 3.3: Gemini AI Integration - Spec Analysis

**Description:** Implement spec analysis using Gemini API

**TDD Requirements:**
- **Test Strategy**: Unit tests with mocked API responses, integration tests with real API
- **Success Criteria**: Analyze PRD content, return clarity score and issues
- **Test Coverage Target**: 90%
- **Key Test Cases**:
  - `analyze_spec()` returns structured analysis
  - Ambiguous phrases detected correctly
  - Clarity score calculated (0-100)
  - API errors handled gracefully
  - Token limits respected
  - Privacy mode anonymizes content
- **Verification Method**: Unit tests + real API calls with sample PRDs

**Implementation Steps:**
1. 🔴 Write tests for `GeminiClient::analyze_spec()`
2. 🟢 Implement basic API call
3. 🔴 Write tests for prompt building
4. 🟢 Implement analysis prompt template
5. 🔴 Write tests for response parsing
6. 🟢 Implement JSON response parser
7. 🔴 Write tests for privacy mode
8. 🟢 Implement content anonymization
9. 🔵 Refactor error handling and retries
10. ✅ Test with real Gemini API

**Files:**
- `src/integrations/ai/gemini.rs`
- `src/integrations/ai/models.rs`
- `src/integrations/ai/gemini_tests.rs`

---

### Task 3.4: Monitoring Integration - Metrics Repository

**Description:** Implement Grafana/Datadog metrics fetching

**TDD Requirements:**
- **Test Strategy**: Unit tests with mocked responses, integration tests with demo instances
- **Success Criteria**: Fetch current metrics, detect active incidents
- **Test Coverage Target**: 90%
- **Key Test Cases**:
  - `get_current_metrics()` returns valid metrics
  - Threshold detection works (Green/Amber/Red)
  - Multiple services monitored concurrently
  - API authentication works
  - Query DSL builds correctly
- **Verification Method**: Unit tests + demo Grafana instance tests

**Implementation Steps:**
1. 🔴 Write tests for `MetricsRepository` trait
2. 🟢 Define interface
3. 🔴 Write tests for Grafana client
4. 🟢 Implement `GrafanaClient`
5. 🔴 Write tests for threshold detection
6. 🟢 Implement threshold logic
7. 🔵 Refactor query builders
8. ✅ Integration tests with demo instances

**Files:**
- `src/integrations/monitoring/grafana.rs`
- `src/integrations/monitoring/models.rs`
- `src/integrations/monitoring/grafana_tests.rs`

---

## Phase 4: Service Layer - Business Logic

### Task 4.1: Search Service - Universal Search

**Description:** Implement universal search with query detection and multi-source aggregation

**TDD Requirements:**
- **Test Strategy**: Unit tests with mocked repositories, integration tests with real data
- **Success Criteria**: Detect query type, search across sources, aggregate results
- **Test Coverage Target**: 95%
- **Key Test Cases**:
  - Ticket ID pattern detected (e.g., "PROJ-123")
  - PR URL pattern detected
  - Generic text searches all sources
  - Results limited to 8 items
  - Concurrent searches aggregate correctly
  - Cache integration reduces API calls
- **Verification Method**: Unit tests + end-to-end search scenarios

**Implementation Steps:**
1. 🔴 Write tests for query type detection
2. 🟢 Implement pattern matching
3. 🔴 Write tests for ticket ID search
4. 🟢 Implement single-source search
5. 🔴 Write tests for multi-source aggregation
6. 🟢 Implement concurrent search with `tokio::join!`
7. 🔴 Write tests for caching behavior
8. 🟢 Integrate cache service
9. 🔵 Refactor result ranking
10. ✅ End-to-end search tests

**Files:**
- `src/services/search_service.rs`
- `src/services/search_service_tests.rs`

---

### Task 4.2: PR Service - Radar Panel Logic

**Description:** Implement PR aggregation with stale detection and filtering

**TDD Requirements:**
- **Test Strategy**: Unit tests for filtering logic, integration tests with mock PRs
- **Success Criteria**: Filter by reviewer, detect stale PRs (>48h), apply custom filters
- **Test Coverage Target**: 95%
- **Key Test Cases**:
  - `get_pending_reviews()` returns only user's PRs
  - Stale detection calculates correctly
  - Filter by repository works
  - Filter by state (Open/Draft) works
  - Time range filtering works
  - Cache fallback on API failure
- **Verification Method**: Unit tests + integration tests

**Implementation Steps:**
1. 🔴 Write tests for `get_pending_reviews()`
2. 🟢 Implement basic PR fetching
3. 🔴 Write tests for stale detection
4. 🟢 Implement stale calculation (>48h default)
5. 🔴 Write tests for filters (repo, state, time)
6. 🟢 Implement filter application
7. 🔴 Write tests for cache fallback
8. 🟢 Implement graceful degradation
9. 🔵 Refactor filter composition
10. ✅ Full integration tests

**Files:**
- `src/services/pr_service.rs`
- `src/services/pr_service_tests.rs`

---

### Task 4.3: Incident Service - Metric Monitoring

**Description:** Implement incident detection and metric threshold monitoring

**TDD Requirements:**
- **Test Strategy**: Unit tests for threshold logic, integration tests with mock metrics
- **Success Criteria**: Detect threshold breaches, aggregate incidents, update tray state
- **Test Coverage Target**: 95%
- **Key Test Cases**:
  - Metric above threshold triggers Amber state
  - Critical metric triggers Red state
  - Multiple services monitored concurrently
  - Tray state calculated correctly
  - Alert history tracked
- **Verification Method**: Unit tests + simulated metric scenarios

**Implementation Steps:**
1. 🔴 Write tests for threshold evaluation
2. 🟢 Implement threshold logic
3. 🔴 Write tests for tray state calculation
4. 🟢 Implement state aggregation (worst-case)
5. 🔴 Write tests for concurrent monitoring
6. 🟢 Implement background polling
7. 🔵 Refactor event publishing
8. ✅ Full scenario tests

**Files:**
- `src/services/incident_service.rs`
- `src/services/incident_service_tests.rs`

---

### Task 4.4: Spec Service - PRD Analysis

**Description:** Implement spec analysis with caching and privacy modes

**TDD Requirements:**
- **Test Strategy**: Unit tests for anonymization, integration tests with Gemini
- **Success Criteria**: Analyze content, cache results, anonymize in privacy mode
- **Test Coverage Target**: 90%
- **Key Test Cases**:
  - `analyze_spec()` returns full analysis
  - Privacy mode anonymizes names, emails, IPs
  - Cache hit avoids API call
  - Analysis result structured correctly
  - Token limit honored
- **Verification Method**: Unit tests + real analysis tests

**Implementation Steps:**
1. 🔴 Write tests for content anonymization
2. 🟢 Implement regex-based anonymization
3. 🔴 Write tests for cache integration
4. 🟢 Implement caching with TTL=1h
5. 🔴 Write tests for privacy mode
6. 🟢 Implement privacy flag handling
7. 🔵 Refactor for readability
8. ✅ Real PRD analysis tests

**Files:**
- `src/services/spec_service.rs`
- `src/services/spec_service_tests.rs`

---

## Phase 5: Frontend - Vue.js Components

### Task 5.1: Flight Console Component

**Description:** Build main command overlay with search and detail panel

**TDD Requirements:**
- **Test Strategy**: Component tests with Vue Test Utils, E2E tests with Playwright
- **Success Criteria**: Opens with hotkey, searches work, keyboard nav functional
- **Test Coverage Target**: 85%
- **Key Test Cases**:
  - Component renders correctly
  - Input autofocuses on open
  - ESC key closes console
  - Arrow keys navigate results
  - Enter opens detail panel
  - Search debounced at 250ms
- **Verification Method**: Component tests + E2E automation

**Implementation Steps:**
1. 🔴 Write component tests for rendering
2. 🟢 Create `FlightConsole.vue` scaffold
3. 🔴 Write tests for keyboard navigation
4. 🟢 Implement keyboard event handlers
5. 🔴 Write tests for search integration
6. 🟢 Integrate with backend search command
7. 🔴 Write tests for detail panel
8. 🟢 Implement sliding detail card
9. 🔵 Refactor styling and animations
10. ✅ E2E tests with Playwright

**Files:**
- `src/components/flight-console/FlightConsole.vue`
- `src/components/flight-console/__tests__/FlightConsole.spec.ts`

---

### Task 5.2: PR Radar Component

**Description:** Build PR monitoring panel with filters and detail view

**TDD Requirements:**
- **Test Strategy**: Component tests for filtering, integration tests for backend calls
- **Success Criteria**: Display PRs, filters work, stale PRs highlighted
- **Test Coverage Target**: 85%
- **Key Test Cases**:
  - PR list renders correctly
  - Filters update PR list
  - Stale PRs show badge
  - Click PR opens detail
  - Actions (copy link, open browser) work
- **Verification Method**: Component tests + manual testing

**Implementation Steps:**
1. 🔴 Write tests for PR list rendering
2. 🟢 Create `PrRadar.vue` component
3. 🔴 Write tests for filter chips
4. 🟢 Implement filter UI
5. 🔴 Write tests for stale detection display
6. 🟢 Implement stale badge logic
7. 🔵 Refactor for performance
8. ✅ Full component tests

**Files:**
- `src/components/pr-radar/PrRadar.vue`
- `src/components/pr-radar/__tests__/PrRadar.spec.ts`

---

### Task 5.3: Spec Scanner Component

**Description:** Build spec analysis interface with input and results display

**TDD Requirements:**
- **Test Strategy**: Component tests for UI logic, integration tests for analysis
- **Success Criteria**: Accept text/URL input, display analysis results, clarity score animates
- **Test Coverage Target**: 80%
- **Key Test Cases**:
  - Text input mode works
  - URL input mode works
  - Analyze button triggers backend
  - Clarity score animates 0→final
  - Results display correctly
  - Error states shown properly
- **Verification Method**: Component tests + manual analysis

**Implementation Steps:**
1. 🔴 Write tests for input modes
2. 🟢 Create `SpecScanner.vue` with mode toggle
3. 🔴 Write tests for analysis trigger
4. 🟢 Implement backend integration
5. 🔴 Write tests for results display
6. 🟢 Implement results UI
7. 🔵 Refactor animations
8. ✅ Full analysis workflow test

**Files:**
- `src/components/spec-scanner/SpecScanner.vue`
- `src/components/spec-scanner/__tests__/SpecScanner.spec.ts`

---

### Task 5.4: Hangar Settings Component

**Description:** Build settings interface for integrations and preferences

**TDD Requirements:**
- **Test Strategy**: Component tests for form validation, integration tests for config saving
- **Success Criteria**: Save/load settings, validate inputs, test connections
- **Test Coverage Target**: 85%
- **Key Test Cases**:
  - Form renders with current settings
  - Input validation works (URLs, tokens)
  - Save button updates backend
  - Test connection buttons work
  - Panic wipe confirms before executing
- **Verification Method**: Component tests + manual settings testing

**Implementation Steps:**
1. 🔴 Write tests for settings load
2. 🟢 Create `Hangar.vue` with tabs
3. 🔴 Write tests for form validation
4. 🟢 Implement validation rules
5. 🔴 Write tests for save functionality
6. 🟢 Integrate with backend config
7. 🔵 Refactor form components
8. ✅ Full settings workflow test

**Files:**
- `src/components/hangar/Hangar.vue`
- `src/components/hangar/__tests__/Hangar.spec.ts`

---

## Phase 6: State Management & IPC

### Task 6.1: Pinia Stores - Frontend State

**Description:** Implement Pinia stores for all major features

**TDD Requirements:**
- **Test Strategy**: Store tests with `setActivePinia`, action tests
- **Success Criteria**: Stores manage state correctly, actions update state properly
- **Test Coverage Target**: 90%
- **Key Test Cases**:
  - Initial state correct
  - Actions update state
  - Getters compute correctly
  - Async actions handle errors
  - State persists as expected
- **Verification Method**: Store unit tests

**Implementation Steps:**
1. 🔴 Write tests for `useFlightConsoleStore`
2. 🟢 Implement Flight Console store
3. 🔴 Write tests for `usePrRadarStore`
4. 🟢 Implement PR Radar store
5. 🔴 Write tests for other stores
6. 🟢 Implement remaining stores
7. 🔵 Refactor for consistency
8. ✅ All store tests pass

**Files:**
- `src/stores/*.ts`
- `src/stores/__tests__/*.spec.ts`

---

### Task 6.2: Tauri Commands - IPC Layer

**Description:** Implement Tauri command handlers for all backend functionality

**TDD Requirements:**
- **Test Strategy**: Integration tests for each command, error handling tests
- **Success Criteria**: Frontend can invoke all backend functions via IPC
- **Test Coverage Target**: 90%
- **Key Test Cases**:
  - `search_command` returns results
  - `get_pending_prs` returns filtered PRs
  - `analyze_spec` returns analysis
  - `save_settings` persists config
  - Error responses formatted correctly
- **Verification Method**: Integration tests + E2E tests

**Implementation Steps:**
1. 🔴 Write tests for search commands
2. 🟢 Implement `search_command` handler
3. 🔴 Write tests for PR commands
4. 🟢 Implement PR-related handlers
5. 🔴 Write tests for other commands
6. 🟢 Implement remaining handlers
7. 🔵 Refactor error serialization
8. ✅ Full IPC integration tests

**Files:**
- `src/commands/*.rs`
- `src/commands/*_tests.rs`

---

## Phase 7: Background Tasks & Polling

### Task 7.1: Background Poller - Async Tasks

**Description:** Implement background polling for PRs and incidents

**TDD Requirements:**
- **Test Strategy**: Unit tests with mock services, integration tests with timers
- **Success Criteria**: Polls at correct intervals, publishes events, handles errors
- **Test Coverage Target**: 90%
- **Key Test Cases**:
  - Polling starts on app launch
  - PR polling executes every N minutes
  - Incident polling executes every M seconds
  - Events published on state changes
  - Polling continues on errors
- **Verification Method**: Unit tests with mock time + manual testing

**Implementation Steps:**
1. 🔴 Write tests for polling initialization
2. 🟢 Implement `BackgroundPoller` struct
3. 🔴 Write tests for PR polling
4. 🟢 Implement PR poll task
5. 🔴 Write tests for incident polling
6. 🟢 Implement incident poll task
7. 🔴 Write tests for error handling
8. 🟢 Implement retry logic
9. 🔵 Refactor for testability
10. ✅ Integration tests with real services

**Files:**
- `src/services/background_poller.rs`
- `src/services/background_poller_tests.rs`

---

### Task 7.2: Event Bus - Observer Pattern

**Description:** Implement pub-sub event system for decoupled communication

**TDD Requirements:**
- **Test Strategy**: Unit tests for subscription and publishing
- **Success Criteria**: Events published reach subscribers, thread-safe
- **Test Coverage Target**: 95%
- **Key Test Cases**:
  - Subscribe to event type
  - Publish event notifies subscribers
  - Multiple subscribers receive events
  - Unsubscribe works correctly
  - Thread-safe concurrent publishing
- **Verification Method**: Unit tests + concurrency tests

**Implementation Steps:**
1. 🔴 Write tests for subscription
2. 🟢 Implement `EventBus::subscribe()`
3. 🔴 Write tests for publishing
4. 🟢 Implement `EventBus::publish()`
5. 🔴 Write tests for concurrency
6. 🟢 Add thread safety with `Arc<RwLock>`
7. 🔵 Refactor for performance
8. ✅ Stress tests

**Files:**
- `src/core/events.rs`
- `src/core/events_tests.rs`

---

## Phase 8: UI/UX Polish & Animations

### Task 8.1: Glassmorphism Styling

**Description:** Implement glass effect CSS with theme support

**TDD Requirements:**
- **Test Strategy**: Visual regression tests, accessibility tests
- **Success Criteria**: Glass panels render correctly, themes switch properly
- **Test Coverage Target**: N/A (visual testing)
- **Key Test Cases**:
  - Light theme renders correctly
  - Dark theme renders correctly
  - System theme detection works
  - Reduced transparency mode works
  - Contrast ratios meet WCAG standards
- **Verification Method**: Visual regression + manual review

**Implementation Steps:**
1. 🔴 Write visual tests for glass panels
2. 🟢 Implement CSS variables for glass effect
3. 🔴 Write tests for theme switching
4. 🟢 Implement theme toggle logic
5. 🔵 Refactor for accessibility
6. ✅ Visual regression suite + manual testing

**Files:**
- `src/assets/styles/glass.css`
- `src/assets/styles/variables.css`

---

### Task 8.2: Micro-Interactions & Animations

**Description:** Implement smooth animations for all interactions

**TDD Requirements:**
- **Test Strategy**: Animation timing tests, visual review
- **Success Criteria**: All animations execute smoothly (<300ms)
- **Test Coverage Target**: N/A (visual testing)
- **Key Test Cases**:
  - Flight Console opens in <200ms
  - Results list animates on changes
  - Detail panel slides smoothly
  - Stale PR glow pulses every 8s
  - Tray icon transitions smoothly
- **Verification Method**: Performance monitoring + manual review

**Implementation Steps:**
1. 🔴 Write performance tests for animations
2. 🟢 Implement CSS transitions
3. 🔴 Write tests for timing
4. 🟢 Tune animation durations
5. 🔵 Optimize for performance
6. ✅ Visual review on all platforms

**Files:**
- `src/assets/styles/animations.css`
- `src/composables/useAnimation.ts`

---

## Phase 9: Integration & E2E Testing

### Task 9.1: End-to-End Test Suite

**Description:** Comprehensive E2E tests covering all user flows

**TDD Requirements:**
- **Test Strategy**: Playwright E2E tests for critical user journeys
- **Success Criteria**: All core flows tested and passing
- **Test Coverage Target**: 80% of user flows
- **Key Test Cases**:
  - Global hotkey opens Flight Console
  - Search ticket by ID and view details
  - View pending PRs in Radar Panel
  - Check incident status
  - Analyze spec with Spec Scanner
  - Update settings in Hangar
- **Verification Method**: Automated E2E suite in CI

**Implementation Steps:**
1. 🔴 Write E2E test for Flight Console flow
2. 🟢 Implement test with Playwright
3. 🔴 Write E2E test for PR Radar flow
4. 🟢 Implement PR workflow test
5. 🔴 Write E2E test for other flows
6. 🟢 Implement remaining E2E tests
7. 🔵 Refactor for test stability
8. ✅ Run full E2E suite in CI

**Files:**
- `e2e/flight-console.spec.ts`
- `e2e/pr-radar.spec.ts`
- `e2e/spec-scanner.spec.ts`

---

### Task 9.2: Performance Testing

**Description:** Benchmark critical paths for performance requirements

**TDD Requirements:**
- **Test Strategy**: Performance benchmarks, load testing
- **Success Criteria**: Flight Console opens <300ms, search returns <700ms
- **Test Coverage Target**: All performance requirements validated
- **Key Test Cases**:
  - Hotkey to visible UI <300ms
  - Search query to first result <700ms
  - Background polling doesn't block UI
  - Memory usage stays under 200MB
  - CPU usage <5% when idle
- **Verification Method**: Automated benchmarks + profiling

**Implementation Steps:**
1. 🔴 Write benchmark for console open time
2. 🟢 Optimize startup path
3. 🔴 Write benchmark for search performance
4. 🟢 Optimize search with caching
5. 🔴 Write memory/CPU monitoring tests
6. 🟢 Optimize resource usage
7. 🔵 Profile and refactor bottlenecks
8. ✅ All benchmarks pass requirements

**Files:**
- `benches/performance.rs`
- `e2e/performance.spec.ts`

---

## Phase 10: Documentation & Release

### Task 10.1: API Documentation

**Description:** Document all public APIs and integration points

**TDD Requirements:**
- **Test Strategy**: Doc tests in Rust, example validation
- **Success Criteria**: All public APIs documented, examples run successfully
- **Test Coverage Target**: 100% public API coverage
- **Key Test Cases**:
  - All doc examples compile and run
  - README instructions work
  - Integration guides accurate
- **Verification Method**: `cargo test --doc`

**Implementation Steps:**
1. 🔴 Write doc tests for core APIs
2. 🟢 Add comprehensive doc comments
3. 🔴 Write integration examples
4. 🟢 Create usage guides
5. ✅ Verify all doc tests pass

---

### Task 10.2: Build & Release Pipeline

**Description:** Setup CI/CD for automated testing and releases

**TDD Requirements:**
- **Test Strategy**: CI pipeline tests, release validation
- **Success Criteria**: Automated builds, tests, and releases work
- **Test Coverage Target**: All platforms build successfully
- **Key Test Cases**:
  - CI runs all tests on push
  - Builds succeed on macOS, Windows, Linux
  - Release artifacts created correctly
  - Code signing works
- **Verification Method**: CI pipeline execution

**Implementation Steps:**
1. 🔴 Write CI config test
2. 🟢 Setup GitHub Actions workflow
3. 🔴 Write release build tests
4. 🟢 Configure release pipeline
5. ✅ Test full release cycle

---

## Summary

**Total Tasks:** 39 implementation tasks across 10 phases

**Estimated Timeline:** 12-16 weeks for full implementation

**Team Size:** 3-4 engineers (2 backend, 1-2 frontend)

**Testing Philosophy:** RED-GREEN-REFACTOR-VERIFY on every task

**Success Metrics:**
- Code coverage: >90% for backend, >85% for frontend
- All E2E tests passing
- Performance requirements met
- Zero critical security issues

---

## Task Execution Order

Follow phases sequentially, but tasks within phases can be parallelized by:
- **Backend Engineer 1:** Phases 1-3 (Infrastructure + Integrations)
- **Backend Engineer 2:** Phase 4 + 7 (Services + Background)
- **Frontend Engineer:** Phases 5-6 (Components + State)
- **Full Team:** Phases 8-10 (Polish + Testing + Release)
