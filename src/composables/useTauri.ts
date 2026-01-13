/**
 * Composable for Tauri backend communication
 * Provides typed wrappers around Tauri invoke commands
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  SearchResponse,
  PrSummary,
  PrItem,
  IncidentSummary,
  IncidentItem,
  Settings,
  CommandError,
  PrGroup,
} from '../types';

// ===== Search Commands =====

export interface SearchParams {
  query: string;
  types?: string[];
  limit?: number;
  includeClosed?: boolean;
}

export async function search(params: SearchParams): Promise<SearchResponse> {
  return invoke<SearchResponse>('search', { params });
}

export async function searchFresh(params: SearchParams): Promise<SearchResponse> {
  return invoke<SearchResponse>('search_fresh', { params });
}

export async function getRecentSearches(): Promise<string[]> {
  return invoke<string[]>('get_recent_searches');
}

export async function clearSearchHistory(): Promise<void> {
  return invoke<void>('clear_search_history');
}

// ===== PR Commands =====

export interface PrListParams {
  repositories?: string[];
  staleOnly?: boolean;
  pendingReviewOnly?: boolean;
  limit?: number;
}

export async function getPrSummary(): Promise<PrSummary> {
  return invoke<PrSummary>('get_pr_summary');
}

export async function getPrs(params: PrListParams = {}): Promise<PrItem[]> {
  return invoke<PrItem[]>('get_prs', { params });
}

export async function getPendingReviewPrs(): Promise<PrItem[]> {
  return invoke<PrItem[]>('get_pending_review_prs');
}

export async function getStalePrs(): Promise<PrItem[]> {
  return invoke<PrItem[]>('get_stale_prs');
}

export async function getPrsGroupedByRepo(): Promise<{ groups: PrGroup[]; totalCount: number }> {
  return invoke('get_prs_grouped_by_repo');
}

export async function getPrsGroupedByAge(): Promise<{ groups: PrGroup[]; totalCount: number }> {
  return invoke('get_prs_grouped_by_age');
}

export async function refreshPrs(): Promise<PrSummary> {
  return invoke<PrSummary>('refresh_prs');
}

// ===== Incident Commands =====

export interface IncidentFilterParams {
  services?: string[];
  minSeverity?: string;
  activeOnly?: boolean;
}

export async function getIncidentSummary(): Promise<IncidentSummary> {
  return invoke<IncidentSummary>('get_incident_summary');
}

export async function getIncidents(params?: IncidentFilterParams): Promise<IncidentItem[]> {
  return invoke<IncidentItem[]>('get_incidents', { params });
}

export async function getCriticalIncidents(): Promise<IncidentItem[]> {
  return invoke<IncidentItem[]>('get_critical_incidents');
}

export async function hasCriticalIncidents(): Promise<boolean> {
  return invoke<boolean>('has_critical_incidents');
}

export async function acknowledgeIncident(incidentId: string): Promise<void> {
  return invoke<void>('acknowledge_incident', { incidentId });
}

export async function refreshIncidents(): Promise<IncidentSummary> {
  return invoke<IncidentSummary>('refresh_incidents');
}

// ===== Settings Commands =====

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

export async function saveJiraConfig(config: {
  baseUrl: string;
  username: string;
  defaultProject?: string;
  hasToken: boolean;
}): Promise<void> {
  return invoke<void>('save_jira_config', { config });
}

export async function saveGitConfig(config: {
  provider: string;
  baseUrl?: string;
  workspace?: string;
  username: string;
  repositories: string[];
  hasToken: boolean;
}): Promise<void> {
  return invoke<void>('save_git_config', { config });
}

export async function saveCredential(credentialType: string, value: string): Promise<void> {
  return invoke<void>('save_credential', { request: { credentialType, value } });
}

export async function deleteCredential(credentialType: string): Promise<void> {
  return invoke<void>('delete_credential', { credentialType });
}

export async function hasCredential(credentialType: string): Promise<boolean> {
  return invoke<boolean>('has_credential', { credentialType });
}

export async function saveShortcuts(shortcuts: {
  flightConsole: string;
  radarPanel: string;
  incidentRadar: string;
}): Promise<void> {
  return invoke<void>('save_shortcuts', { shortcuts });
}

export async function saveAppearance(appearance: {
  theme: string;
  glassIntensity: number;
  reduceTransparency: boolean;
}): Promise<void> {
  return invoke<void>('save_appearance', { appearance });
}

export async function testConnection(integration: string): Promise<boolean> {
  return invoke<boolean>('test_connection', { integration });
}

export async function panicWipe(): Promise<number> {
  return invoke<number>('panic_wipe');
}

// ===== Utility =====

export function isCommandError(error: unknown): error is CommandError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error
  );
}
