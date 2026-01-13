<template>
  <div class="search-results">
    <div v-if="loading" class="results-loading">
      <div class="skeleton" v-for="i in 3" :key="i"></div>
    </div>

    <div v-else-if="results.length === 0 && query" class="results-empty">
      <div class="empty-icon">🔍</div>
      <p>No results found for "{{ query }}"</p>
      <p class="empty-hint">Try a different search term or check the filters</p>
    </div>

    <div v-else-if="results.length === 0 && !query" class="results-placeholder">
      <p>Start typing to search across tickets, PRs, and documents</p>
      <div class="quick-actions">
        <button class="quick-action" @click="$emit('action', 'my-tickets')">
          <span class="quick-icon">🎫</span>
          My Tickets
        </button>
        <button class="quick-action" @click="$emit('action', 'pending-reviews')">
          <span class="quick-icon">🔀</span>
          Pending Reviews
        </button>
        <button class="quick-action" @click="$emit('action', 'incidents')">
          <span class="quick-icon">🚨</span>
          Active Incidents
        </button>
      </div>
    </div>

    <div v-else class="results-list" role="listbox">
      <div
        v-for="(result, index) in results"
        :key="result.id"
        :class="['result-item', { 'result-item--selected': selectedIndex === index }]"
        role="option"
        :aria-selected="selectedIndex === index"
        @click="$emit('select', result)"
        @mouseenter="$emit('hover', index)"
      >
        <div class="result-icon">{{ result.icon }}</div>
        
        <div class="result-content">
          <div class="result-title">{{ result.title }}</div>
          <div v-if="result.subtitle" class="result-subtitle">{{ result.subtitle }}</div>
        </div>

        <div class="result-meta">
          <span v-if="result.metadata.status" :class="['status-badge', getStatusClass(result.metadata.status)]">
            {{ result.metadata.status }}
          </span>
          <span v-if="result.metadata.isStale" class="stale-indicator" title="Stale">⏰</span>
        </div>

        <div class="result-type">
          <span class="type-badge">{{ result.type }}</span>
        </div>
      </div>
    </div>

    <div v-if="results.length > 0" class="results-footer">
      <span class="results-count">{{ results.length }} results</span>
      <div class="keyboard-hints">
        <span><kbd>↑↓</kbd> Navigate</span>
        <span><kbd>↵</kbd> Open</span>
        <span><kbd>Esc</kbd> Close</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SearchResult } from '../../types';

interface Props {
  results: SearchResult[];
  selectedIndex: number;
  loading?: boolean;
  query?: string;
}

defineProps<Props>();

defineEmits<{
  'select': [result: SearchResult];
  'hover': [index: number];
  'action': [action: string];
}>();

function getStatusClass(status: string): string {
  const lower = status.toLowerCase();
  if (lower.includes('done') || lower.includes('closed') || lower.includes('merged')) {
    return 'status-badge--green';
  }
  if (lower.includes('progress') || lower.includes('review')) {
    return 'status-badge--amber';
  }
  if (lower.includes('blocked') || lower.includes('critical')) {
    return 'status-badge--red';
  }
  return 'status-badge--neutral';
}
</script>

<style scoped>
.search-results {
  max-height: 400px;
  overflow-y: auto;
}

.results-loading {
  padding: var(--space-4);
}

.skeleton {
  height: 60px;
  background: linear-gradient(90deg, var(--glass-bg) 25%, var(--glass-bg-hover) 50%, var(--glass-bg) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--radius-md);
  margin-bottom: var(--space-2);
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

.results-empty,
.results-placeholder {
  padding: var(--space-8) var(--space-4);
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 2rem;
  margin-bottom: var(--space-3);
}

.empty-hint {
  font-size: var(--text-sm);
  color: var(--text-muted);
  margin-top: var(--space-2);
}

.quick-actions {
  display: flex;
  gap: var(--space-3);
  justify-content: center;
  margin-top: var(--space-4);
}

.quick-action {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.quick-action:hover {
  background: var(--glass-bg-hover);
  color: var(--text-primary);
  transform: translateY(-1px);
}

.quick-icon {
  font-size: 1.2rem;
}

.results-list {
  padding: var(--space-2);
}

.result-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.result-item:hover,
.result-item--selected {
  background: var(--glass-bg-hover);
}

.result-item--selected {
  background: var(--glass-bg-active);
}

.result-icon {
  font-size: 1.25rem;
  width: 32px;
  text-align: center;
}

.result-content {
  flex: 1;
  min-width: 0;
}

.result-title {
  font-weight: var(--font-medium);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-subtitle {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.status-badge {
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  border-radius: var(--radius-full);
  background: var(--glass-bg);
}

.status-badge--green { background: rgba(34, 197, 94, 0.2); color: #22c55e; }
.status-badge--amber { background: rgba(245, 158, 11, 0.2); color: #f59e0b; }
.status-badge--red { background: rgba(239, 68, 68, 0.2); color: #ef4444; }
.status-badge--neutral { background: var(--glass-bg); color: var(--text-secondary); }

.stale-indicator {
  font-size: var(--text-sm);
}

.result-type {
  margin-left: var(--space-2);
}

.type-badge {
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-muted);
  background: var(--glass-bg);
  border-radius: var(--radius-sm);
}

.results-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--glass-border);
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.results-count {
  font-weight: var(--font-medium);
}

.keyboard-hints {
  display: flex;
  gap: var(--space-4);
}

.keyboard-hints kbd {
  padding: var(--space-1);
  font-size: var(--text-xs);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-sm);
  margin-right: var(--space-1);
}
</style>
