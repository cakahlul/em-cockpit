/* TypeScript type definitions for the EM Cockpit frontend */

// ===== Search Types =====
export type SearchResultType = 'Ticket' | 'PR' | 'Incident' | 'Document';

export interface SearchMetadata {
  status?: string;
  assignee?: string;
  priority?: string;
  isStale?: boolean;
}

export interface SearchResult {
  id: string;
  type: SearchResultType;
  icon: string;
  title: string;
  subtitle?: string;
  url?: string;
  score: number;
  metadata: SearchMetadata;
}

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  query: string;
}

// ===== PR Types =====
export interface User {
  id: string;
  name: string;
  avatar?: string;
}

export interface Reviewer {
  user: User;
  approved: boolean;
}

export interface PrItem {
  id: string;
  repository: string;
  title: string;
  description?: string;
  state: string;
  author: User;
  reviewers: Reviewer[];
  sourceBranch: string;
  targetBranch: string;
  checksStatus: string;
  isStale: boolean;
  updatedAt: string;
  url: string;
  ageHours: number;
}

export interface PrSummary {
  totalOpen: number;
  pendingReview: number;
  staleCount: number;
  byRepository: Record<string, number>;
  trayState: TrayState;
}

export interface PrGroup {
  label: string;
  prs: PrItem[];
  staleCount: number;
}

// ===== Incident Types =====
export type Severity = 'low' | 'medium' | 'high' | 'critical';

export interface IncidentItem {
  id: string;
  service: string;
  severity: Severity;
  severityLevel: number;
  status: string;
  description: string;
  startedAt: string;
  resolvedAt?: string;
  durationMins: number;
  runbookUrl?: string;
}

export interface IncidentSummary {
  totalActive: number;
  criticalCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
  byService: Record<string, number>;
  trayState: TrayState;
  mostSevere?: Severity;
}

// ===== Settings Types =====
export interface JiraConfig {
  baseUrl: string;
  username: string;
  defaultProject?: string;
  hasToken: boolean;
}

export interface GitConfig {
  provider: string;
  baseUrl?: string;
  workspace?: string;
  username: string;
  repositories: string[];
  hasToken: boolean;
}

export interface GeminiConfig {
  model: string;
  hasApiKey: boolean;
}

export interface GrafanaConfig {
  baseUrl: string;
  services: string[];
  hasApiKey: boolean;
}

export interface IntegrationConfig {
  jira?: JiraConfig;
  git?: GitConfig;
  gemini?: GeminiConfig;
  grafana?: GrafanaConfig;
}

export interface ShortcutConfig {
  flightConsole: string;
  radarPanel: string;
  incidentRadar: string;
}

export interface AppearanceConfig {
  theme: 'light' | 'dark' | 'system';
  glassIntensity: number;
  reduceTransparency: boolean;
}

export interface Settings {
  integrations: IntegrationConfig;
  shortcuts: ShortcutConfig;
  appearance: AppearanceConfig;
  prStaleThresholdHours: number;
}

// ===== Tray State =====
export type TrayState = 'neutral' | 'green' | 'amber' | 'red';

// ===== Command Errors =====
export interface CommandError {
  code: string;
  message: string;
}
