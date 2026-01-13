/**
 * Vue Component Test Setup
 * 
 * Configuration and utilities for Vue component testing.
 */

import { config } from '@vue/test-utils';

// Mock Tauri API
const mockInvoke = vi.fn().mockImplementation((cmd: string, args?: any) => {
  // Default mock responses for Tauri commands
  switch (cmd) {
    case 'search':
      return Promise.resolve({
        results: [],
        total: 0,
        query: args?.params?.query || '',
      });
    
    case 'get_pr_summary':
      return Promise.resolve({
        totalOpen: 0,
        pendingReview: 0,
        staleCount: 0,
        byRepository: {},
        trayState: 'neutral',
      });
    
    case 'get_prs':
      return Promise.resolve([]);
    
    case 'get_incident_summary':
      return Promise.resolve({
        totalActive: 0,
        criticalCount: 0,
        highCount: 0,
        mediumCount: 0,
        lowCount: 0,
        byService: {},
        trayState: 'green',
        mostSevere: null,
      });
    
    case 'get_settings':
      return Promise.resolve({
        integrations: {},
        shortcuts: {
          flightConsole: 'Alt+Space',
          radarPanel: 'Ctrl+2',
          incidentRadar: 'Ctrl+3',
        },
        appearance: {
          theme: 'dark',
          glassIntensity: 0.8,
          reduceTransparency: false,
        },
        prStaleThresholdHours: 48,
      });
    
    default:
      return Promise.resolve(null);
  }
});

// Global mocks
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Configure Vue Test Utils globally
config.global.stubs = {
  Teleport: true,
  Transition: false,
  TransitionGroup: false,
};

// Export mock for test files to customize
export { mockInvoke };

// Test utilities
export function createMockSearchResult(overrides = {}) {
  return {
    id: 'TEST-123',
    type: 'Ticket',
    icon: '🎫',
    title: 'Test Result',
    subtitle: 'Test subtitle',
    url: 'https://example.com',
    score: 1.0,
    metadata: {
      status: 'Open',
      assignee: 'Test User',
      priority: 'Medium',
      isStale: false,
    },
    ...overrides,
  };
}

export function createMockPrItem(overrides = {}) {
  return {
    id: '123',
    repository: 'test-repo',
    title: 'Test PR',
    description: 'Test description',
    state: 'open',
    author: {
      id: 'user1',
      name: 'Test Author',
      avatar: null,
    },
    reviewers: [],
    sourceBranch: 'feature',
    targetBranch: 'main',
    checksStatus: 'pass',
    isStale: false,
    updatedAt: new Date().toISOString(),
    url: 'https://example.com/pr/123',
    ageHours: 12,
    ...overrides,
  };
}

export function createMockIncidentItem(overrides = {}) {
  return {
    id: 'inc-1',
    service: 'api-service',
    severity: 'medium',
    severityLevel: 2,
    status: 'firing',
    description: 'Test incident',
    startedAt: new Date().toISOString(),
    resolvedAt: null,
    durationMins: 30,
    runbookUrl: null,
    ...overrides,
  };
}

// Wait for async operations
export async function flushPromises() {
  return new Promise(resolve => setTimeout(resolve, 0));
}

// Setup global test environment
beforeEach(() => {
  mockInvoke.mockClear();
});
