# The EM Cockpit – Product Requirements Document

## 1. Product Overview

### 1.1 Product Name

- **The EM Cockpit**


### 1.2 Elevator Pitch

- A spotlight-style **cockpit** desktop app that lets Engineering Managers command their work from anywhere (tickets, PRs, incidents, specs) via a global hotkey, without opening a browser.
- It reduces context switching by letting EMs query tickets, PRs, incidents, and specs from a single global overlay, while keeping focus on the current deep work window.


### 1.3 Target Users

- **Primary**
    - Engineering Managers in software teams (5–50 engineers).
    - Tech Leads who frequently handle EM responsibilities.
- **Secondary**
    - Staff/Principal Engineers acting as de-facto EMs.


### 1.4 Platforms \& Technology

- **Platforms**
    - Desktop: macOS, Windows, Linux (Tauri-based desktop app).
- **Core Stack**
    - Rust (backend/core):
        - Global shortcut listener.
        - System tray lifecycle and status.
        - Secure storage of integration credentials.
        - Async HTTP polling and aggregation for:
            - Jira APIs.
            - Git hosting (Bitbucket).
            - Documentation (Confluence).
            - Observability platforms (Grafana/Datadog).
        - Integration with Gemini API for PRD/spec analysis.
    - JavaScript Frontend (Vue.js):
        - Renders search overlay (Flight Console), PR radar, incident radar, and spec scanner.
        - Manages state, navigation, and UI animations.


### 1.5 Non‑Goals (v1)

- No full-featured editing of tickets or PRs inside the app (view + quick actions only).
- No low-code automation builder or workflow designer.
- No mobile or web/browser version.
- No multi-tenant team dashboard (v1 is per-user cockpit).

***

## 2. Objectives \& Success Metrics

### 2.1 Objectives

- Reduce context switching and time to answer simple questions (status, assignee, link, PRs pending review, incident status).
- Improve responsiveness to:
    - Stale PRs that need EM review.
    - Active incidents and error spikes.
- Improve PRD/spec quality before grooming sessions and implementation.


### 2.2 Success Metrics (Post-Launch v1)

- Time to get “ticket status + assignee + link” reduced by ≥ 50% vs. browser-based flow (self-reported).
- ≥ 80% of EMs fire the cockpit at least 5 times per day after 2 weeks of adoption.
- ≥ 30% of PRs reviewed using the PR radar / Radar Panel.
- ≥ 50% of PRDs in a sprint have been run through Spec Scanner before grooming.

***

## 3. Core User Flows

### 3.1 Global Command Flow (Flight Console)

**Scenario**

An EM is coding or in a doc, receives a ping about `PROJ-123`, wants the status without opening a browser.

**Steps**

1. User presses global hotkey (default: `Alt + Space` / `Option + Space`).
2. The **Flight Console** (spotlight-style overlay) appears centered on screen, dimming the background.
3. Command input is focused automatically.
4. User types:
    - Ticket ID (e.g., `PROJ-123`), or
    - PR link/ID, or
    - Command such as `my prs`, `incidents`, `spec scanner`.
5. Results appear in a glassmorphism list under the command input:
    - Tickets, PRs, specs, incidents depending on query.
6. User navigates via arrow keys or mouse:
    - Presses `Enter` to open a detail card in the same overlay (no browser).
    - Triggers quick actions (copy link, open in browser).
7. User presses `Esc` or clicks outside to close the Flight Console and returns to the original app.

**Success Criteria**

- Time from hotkey press to visible console: < 300 ms (excluding network).
- Time from hotkey press to first result: < 700 ms on a standard connection.
- Typical “status check + dismiss” interaction completes in ≤ 10 seconds.

***

## 4. Feature Set \& Cockpit Taxonomy

### 4.1 Cockpit Terminology

To emphasize the **cockpit** metaphor:

- **Flight Console**: Global command/search overlay (main spotlight).
- **Radar Panel**: PR-focused view (PR Watch).
- **Incident Radar**: SRE Bridge view for incidents and metrics.
- **Spec Scanner**: PRD Analyzer view (spec linting).
- **Hangar**: Settings, integrations, preferences.
- **HUD (Heads-Up Display)**: Top section of Flight Console showing key status (PR count, incidents, next event).

***

## 4.2 Global Hotkey \& System Tray

### 4.2.1 Global Hotkey (Flight Console Invocation)

**Requirements**

- Configurable global shortcut (default `Alt/Option + Space`).
- Works from any active application (subject to OS restrictions).
- Invokes the Flight Console overlay in the center of the current screen.
- Overlay:
    - Glassmorphism panel (blurred background, soft border, subtle shadow).
    - Respects OS theme (light/dark) with consistent cockpit character.

**Behaviors**

- If Flight Console is already open:
    - Re-pressing the hotkey closes it.
- When open:
    - Background is dimmed with a translucent dark layer.
    - The active app beneath remains visible but defocused.

**Micro-Interactions**

- On open: quick scale-up + fade-in (~150–200 ms).
- On close: reverse animation with a gentle opacity fade-out.


### 4.2.2 System Tray Icon (Tray Beacon)

**States**

- Neutral: No pending PRs requiring user attention and no active incidents.
- Green: All monitored systems nominal and no pending PR reviews.
- Amber:
    - At least one PR pending review > 24 hours, or
    - Upcoming calendar event (future enhancement) – keep placeholder but no integration in v1.
- Red:
    - Active incident or error rate above configured thresholds.

**Behaviors**

- Hover:
    - Tooltip summarizing current state (e.g., “3 PRs waiting. No active incidents.”).
- Click:
    - Single click: open Flight Console.
- Right-click context menu:
    - “Open Flight Console”
    - “Open Radar Panel”
    - “Open Incident Radar”
    - “Open Hangar (Settings)”
    - “Quit”

**Micro-Interactions**

- When an incident crosses a critical threshold:
    - Tray icon briefly pulses/glows red for ~3 seconds.
- Color transitions animate smoothly (~200 ms).

***

## 4.3 Universal Search (Flight Console)

### 4.3.1 Input \& Detection

**Input Types**

- Ticket IDs: matching configured patterns, e.g., `[A-Z]+-\d+`.
- PR references:
    - Direct PR URLs.
    - Numeric IDs or repo-specific patterns.
- Commands (free text) such as:
    - `my prs`
    - `stale prs`
    - `incidents`
    - `spec scanner`
    - `lint spec`

**Detection Logic**

- If input matches ticket pattern → ticket lookup mode.
- If input matches PR URL/ID pattern → PR lookup mode.
- If input matches known commands → navigation mode.
- Otherwise → generic search:
    - Ticket summary search.
    - Document title search (specs).
    - Optional future: fuzzy command suggestions.


### 4.3.2 Search Results List

**Layout**

- Maximum 8 items visible with scroll.
- Each row:
    - Left: icon (ticket, PR, doc, incident).
    - Center:
        - Primary: title/summary (single line).
        - Secondary: status, assignee/author, time since last update.
    - Right:
        - Priority badge (e.g., `P1`, `High`) or state badge (e.g., `Stale 3d`).

**Result Types**

- Ticket:
    - Key, summary, status, assignee, priority, sprint, updated time.
- PR:
    - Repo, title, state, author, requested reviewers, checks status, updated time.
- Spec/Doc:
    - Title, space/workspace, last updated, owner.
- Incident:
    - Service name, severity, started time, current status.

**Interactions**

- Keyboard:
    - Up/Down: move between results.
    - Enter: open detail view for selected item.
- Mouse:
    - Click row: open detail view.

**Micro-Interactions**

- Hover: row slightly raises, backdrop blur intensifies, left edge accent line animates in.
- Selection: row highlight animates with soft color gradient.


### 4.3.3 Detail Panel (Within Flight Console)

**Behavior**

- Appears as a sliding glass card from the right side of the overlay.
- Does not open external browser by default; keeps user in cockpit.

**Ticket Detail Content**

- Header:
    - Ticket key + summary.
- Properties:
    - Status (pill).
    - Assignee (avatar + name if available).
    - Priority.
    - Sprint/iteration.
    - Labels.
    - Last updated relative time.
- Snippets:
    - Last 1–3 comments (if available via API).
- Actions:
    - “Copy link”
    - “Open in browser”
    - “Copy summary”

**PR Detail Content**

- Header:
    - Repo + PR title.
- Properties:
    - State (Open/Merged/Declined).
    - Author.
    - Reviewers.
    - Checks status (Pass/Fail/Running).
    - Time since last activity.
- Snippets:
    - Last review comment excerpts (if accessible).
- Actions:
    - “Copy review link”
    - “Open diff in browser”
    - “Copy branch name”

**Micro-Interactions**

- Detail panel slides in (200–250 ms) with slight blur reveal.
- Action buttons have subtle hover elevation and micro color transitions.

***

## 4.4 PR Watch – Radar Panel

### 4.4.1 Purpose

- Provide a dedicated **Radar Panel** that lists:
    - PRs requiring the user’s review.
    - Stale PRs (no update beyond configured time, default 48 hours).
    - PRs tagged as blocking or marked as important for EM attention.


### 4.4.2 Access

- Command in Flight Console:
    - `radar`, `pr radar`, `my prs`.
- Tray context menu:
    - “Open Radar Panel”.
- HUD:
    - PR badge; clicking opens Radar Panel.


### 4.4.3 Data \& Filters

**Data Sources**

- Git hosting API (Bitbucket/GitHub/GitLab equivalents).

**Filter Logic**

- PRs where:
    - Current user is a requested reviewer or reviewer.
    - OR PR is labeled/annotated as blocking.
- Additional filters:
    - Needs Review (no review from current user).
    - Stale PRs (no activity > `stale_threshold_hours`).
    - Time range: last 7 / 14 / 30 days.
    - Repositories/projects (multi-select).


### 4.4.4 Radar Panel Layout

**Structure**

- Two-column glass layout:
    - Left: PR list.
    - Right: detail view for selected PR.
    - Top bar: filter chips and context info.

**Left Column: PR List**

- Row content:
    - Repo name.
    - PR title.
    - Author.
    - Badges:
        - State (Open, Draft).
        - Check status (Passed, Failing).
        - “Stale” if applicable.
    - Time since last activity (e.g., `3d`, `5h`).

**Right Column: PR Detail**

- Summarizes:
    - Title, repo, branch target/source.
    - Author, reviewers.
    - State, checks.
    - Recent activity (last comments, approvals).
- Actions:
    - “Copy review link”.
    - “Open diff in browser”.
    - “Copy branch name”.

**Micro-Interactions**

- Stale PR rows:
    - Subtle pulsing glow on left accent every ~8 seconds.
- Filter changes:
    - List animates with fade/slide transitions.
- Selecting PR:
    - Detail card slides in with a short motion and opacity increase.

***

## 4.5 SRE Bridge – Incident Radar

### 4.5.1 Purpose

- Provide an **Incident Radar** view to surface:
    - Active incidents.
    - Key metrics (error rate, latency, etc.) at a glance.
- Avoid opening full Grafana/Datadog dashboards during deep work unless absolutely necessary.


### 4.5.2 Access

- Commands:
    - `incidents`, `incident radar`, `metrics`.
- Tray Beacon:
    - Color and tooltip reflect incident state.
    - Context menu item: “Open Incident Radar”.


### 4.5.3 Data \& Metrics

**Data Sources**

- Observability/monitoring tools (Grafana/Datadog-like).
- Incident management (if integrated in future; v1 may fetch active alerts directly from monitoring APIs).

**Configurable in Hangar**

- List of services or dashboards to track.
- Metrics:
    - Error rate.
    - Latency (p95/p99).
    - Saturation (CPU/memory).
- Alert thresholds:
    - Green/Amber/Red boundaries.


### 4.5.4 Incident Radar Layout

**HUD / Top Strip**

- Displays up to 3 key metrics as pill badges, e.g.:
    - `Errors: 0.5%`
    - `Latency p95: 350 ms`
    - `CPU: 60%`

**Main View**

- Incident list:
    - Service/system.
    - Severity (Critical/High/Medium).
    - Started time (relative).
    - Current status (Firing/Resolved).
    - Link to runbook or full dashboard.
- Optional: small sparkline per incident for a key metric (keep visually minimal to preserve clarity).


### 4.5.5 Tray Beacon Behavior for Incidents

- Color changes:
    - Green → Amber → Red based on metrics/alerts.
- Tooltip summarizing:
    - Highest severity incident.
    - Count of active alerts.

**Micro-Interactions**

- Newly detected incident rows:
    - Slide in from bottom with slight fade.
- Beacon color shifts:
    - Smooth transitions, no abrupt flickering.

***

## 4.6 PRD Analyzer – Spec Scanner

### 4.6.1 Purpose

- Provide **Spec Scanner** (PRD Analyzer) to:
    - Detect ambiguous language.
    - Identify missing edge cases.
    - Highlight non-testable requirements.
    - Assign a clarity score.


### 4.6.2 Access

- Commands:
    - `spec scanner`, `lint spec`, `scan spec`.
- Flight Console:
    - Keyboard navigation or command suggestions to jump into Spec Scanner.


### 4.6.3 Input Modes

- Direct text:
    - User pastes PRD/spec text into a text area.
- URL:
    - User pastes a Confluence/Notion-like link.
    - App fetches content via configured credentials/API; user must explicitly trigger fetch.


### 4.6.4 Analysis Behavior

- Backend sends content to Gemini with prompts that enforce:
    - Highlighting ambiguous terms (e.g., “fast”, “user-friendly”, “robust”, “simple”) and suggesting clearer, measurable alternatives.
    - Listing missing edge cases (e.g., error scenarios, boundary conditions, atypical users).
    - Identifying non-testable requirements and suggesting how to make them testable.
    - Returning a clarity score (0–100) and a brief justification.


### 4.6.5 Spec Scanner UI

**Layout**

- Two-column layout:
    - Left:
        - Input area with:
            - Text editor for pasted content.
            - Link input field for URL mode.
            - “Analyze” button.
    - Right:
        - Analysis results:
            - Clarity score as gauge or bar.
            - Sections:
                - Ambiguous Language.
                - Missing Scenarios.
                - Risks \& Open Questions.

**Result Items**

- For each ambiguous phrase:
    - Original text snippet.
    - Explanation of why it is ambiguous.
    - Suggested clearer rewrite.
- For missing scenarios:
    - Scenario description.
    - Impact if not addressed.
- For risks:
    - Risk description.
    - Suggested mitigation or follow-up question.

**Error Handling**

- If AI call fails:
    - Show banner: “Analysis failed: <reason>. Retry / Check API settings.”
- If document fetch fails:
    - “Unable to fetch document. Check URL and permissions.”

**Micro-Interactions**

- On “Analyze” click:
    - Subtle scanning animation across the right panel (progress shimmer).
- Clarity score:
    - Animates from 0 to final value in ~600 ms.

***

## 4.7 Hangar – Settings \& Integrations

### 4.7.1 Purpose

- Central configuration hub (the **Hangar**) for:
    - Integrations.
    - Shortcuts.
    - Appearance.
    - Preferences \& privacy.


### 4.7.2 Sections

**Integrations**

- Jira-like:
    - Base URL.
    - Personal access token.
    - Default project filters.
- Git hosting:
    - Provider selection (Bitbucket, GitHub, GitLab).
    - Personal access token or OAuth token.
- Documentation:
    - Confluence/Notion endpoints, API keys.
- Observability:
    - Grafana/Datadog-like URLs, credentials, metric templates.
- Gemini:
    - API key.
    - Model selection.
    - Optional per-day token/usage limit.

**Shortcuts**

- Global:
    - Flight Console hotkey (with conflict detection).
- In-overlay:
    - Quick navigation shortcuts (e.g., `Ctrl+1` Flight Console, `Ctrl+2` Radar Panel, `Ctrl+3` Incident Radar).

**Appearance**

- Theme:
    - Light / Dark / System.
- Glass intensity:
    - Slider controlling transparency/blur level.
    - Option “Reduce transparency” for accessibility.

**Preferences**

- PR stale threshold:
    - Default 48 hours; configurable.
- Incident thresholds:
    - Per-metric thresholds for Amber/Red.
- Privacy:
    - Option to not store analyzed content history.
    - “Wipe data” / “Panic wipe” to clear tokens and local cache.

***

## 5. UI \& UX Specification

### 5.1 Visual Style – Glass Cockpit

**Principles**

- Modern glassmorphism inspired by macOS:
    - Translucent panels with backdrop blur.
    - Rounded corners and soft shadows.
- Focus on clarity:
    - Limited color palette:
        - Primary accent: electric blue/cyan.
        - Secondary: amber and red for warnings and critical states.
- Depth:
    - Background dimming when an overlay is open.
    - Clear separation between primary (Flight Console) and secondary panels (details, filters).

**Accessibility**

- Ensure sufficient text contrast over glass backgrounds.
- Provide a toggle in Appearance to reduce transparency or use solid backgrounds.


### 5.2 Screen Layouts

**Flight Console**

- Centered frosted glass card.
- Sections:
    - HUD:
        - Top bar with:
            - Left: app name/logo (“The EM Cockpit”).
            - Center: small status indicators (e.g., `3 PRs`, `1 incident`).
            - Right: link to Hangar (settings icon).
    - Command Input:
        - Large, horizontally centered text field.
        - Placeholder: “Command your cockpit… (e.g., PROJ-123, my prs, incidents)”.
    - Results List:
        - Below input, scrollable, glass rows.
    - Detail Panel:
        - Right-sliding card when an item is opened.

**Radar Panel**

- Full-width glass panel with:
    - Top:
        - Filters as chips.
        - Search within PRs.
    - Left:
        - List of PRs.
    - Right:
        - Detail card.

**Incident Radar**

- Similar two-column layout:
    - Left:
        - Incidents list.
    - Right:
        - Selected incident details and metrics.

**Spec Scanner**

- Left:
    - Mode toggle (Text / URL).
    - Input area.
    - Analyze button fixed at bottom or top right of input region.
- Right:
    - Clarity score gauge at top.
    - Scrollable sections for issues.


### 5.3 Interaction Model

**Keyboard Navigation**

- `Esc`:
    - Close current overlay (Flight Console, Radar, Spec Scanner).
- `Tab`:
    - Cycle through:
        - Command input.
        - Results list.
        - Detail panel.
- Arrow keys:
    - Up/Down in results list.
- `Enter`:
    - Execute selected item or submit command.
- `Ctrl+K` / `Cmd+K`:
    - Clear command input.

**Error States**

- Global error:
    - Banner near top of overlay with context and actions (Retry, Open Settings).
- Empty states:
    - Radar Panel:
        - “No PRs need your attention. Enjoy clear skies.”
    - Incident Radar:
        - “All systems nominal.”

***

## 6. Technical \& Data Requirements

### 6.1 Architecture Responsibilities

**Rust (Backend)**

- Global hotkey registration per OS.
- System tray:
    - Icon drawing and state-changing.
    - Context menu actions.
- Secure storage:
    - Tokens and sensitive config in OS keychain or encrypted file.
- Background tasks:
    - Periodic polling of Jira/PR/metrics APIs.
    - Debouncing and caching to reduce latency and rate limit issues.
- AI integration:
    - Communication with Gemini API for Spec Scanner.
    - Respecting configured limits and error handling.

**Frontend (JS – React/Svelte)**

- Rendering:
    - Flight Console.
    - Radar Panel.
    - Incident Radar.
    - Spec Scanner.
    - Hangar.
- State management:
    - Local state for overlays and views.
    - Integration status and error flags.
- Animations \& micro-interactions:
    - Using lightweight animation libraries or native CSS transitions.


### 6.2 Performance Requirements

- Flight Console open:
    - < 300 ms from hotkey to visible UI on typical desktop hardware.
- Search:
    - Debounce user input (~150–250 ms).
    - Show loading skeletons while waiting for API responses.
- Network:
    - Use concurrency where possible.
    - Cache recent results (e.g., last 10 queries, radar lists).


### 6.3 Security \& Privacy

- API keys and tokens:
    - Always stored encrypted.
    - Never logged in plaintext.
- AI data:
    - Only send content when user explicitly triggers analysis.
    - Provide warning/notice about sending proprietary content if needed.
- Data clear:
    - “Panic wipe” in Hangar clears local cache and credentials.

***

## 7. Out of Scope (v1) \& Future Enhancements

### 7.1 Out of Scope (v1)

- PR merge/approve directly from the app.
- Detailed dashboards for team-level metrics.
- Calendar integration in HUD.
- Team-wide shared “cockpit configurations”.


### 7.2 Future Enhancements (Non-Blocking Ideas)

- Saved commands / macros (e.g., “morning sweep”).
- Team cockpit: share default metrics, filters, and Spec Scanner templates.
- Notifications panel with configurable rules.
- Advanced AI capabilities for:
    - Briefing EM on current sprint health.
    - Summarizing PRs or incidents.

