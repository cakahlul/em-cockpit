<template>
  <div class="pr-radar" :class="{ 'pr-radar--loading': loading }">
    <!-- Header -->
    <header class="radar-header glass-panel">
      <div class="header-content">
        <h2 class="radar-title">
          <span class="title-icon">🔀</span>
          PR Radar
        </h2>
        <StatusDot :state="trayState" :title="statusMessage" />
      </div>
      
      <div class="header-stats">
        <div class="stat">
          <span class="stat-value">{{ totalOpen }}</span>
          <span class="stat-label">Open</span>
        </div>
        <div class="stat stat--warning" v-if="staleCount > 0">
          <span class="stat-value">{{ staleCount }}</span>
          <span class="stat-label">Stale</span>
        </div>
        <div class="stat stat--accent" v-if="pendingReview > 0">
          <span class="stat-value">{{ pendingReview }}</span>
          <span class="stat-label">To Review</span>
        </div>
      </div>

      <button class="refresh-btn" @click="refresh" :disabled="loading">
        <span :class="['refresh-icon', { 'animate-spin': loading }]">🔄</span>
      </button>
    </header>

    <!-- Filters -->
    <div class="radar-filters">
      <button
        v-for="filter in filters"
        :key="filter.key"
        :class="['filter-chip', { 'filter-chip--active': activeFilter === filter.key }]"
        @click="setFilter(filter.key)"
      >
        {{ filter.label }}
        <Badge v-if="filter.count > 0" :variant="filter.variant">{{ filter.count }}</Badge>
      </button>
    </div>

    <!-- PR List -->
    <div class="radar-content">
      <div v-if="loading && filteredPrs.length === 0" class="loading-skeleton">
        <div class="skeleton" v-for="i in 4" :key="i"></div>
      </div>

      <div v-else-if="filteredPrs.length === 0" class="empty-state">
        <span class="empty-icon">✨</span>
        <p>No PRs to review</p>
        <p class="empty-subtitle">You're all caught up!</p>
      </div>

      <TransitionGroup v-else name="pr-list" tag="div" class="pr-list">
        <PrCard
          v-for="(pr, index) in filteredPrs"
          :key="pr.id"
          :pr="pr"
          :style="{ animationDelay: `${index * 50}ms` }"
          @click="openPr(pr)"
        />
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import StatusDot from '../common/StatusDot.vue';
import Badge from '../common/Badge.vue';
import PrCard from './PrCard.vue';
import type { PrItem, TrayState } from '../../types';
import { getPrSummary, getPrs, refreshPrs } from '../../composables/useTauri';

// State
const loading = ref(false);
const prs = ref<PrItem[]>([]);
const totalOpen = ref(0);
const staleCount = ref(0);
const pendingReview = ref(0);
const trayState = ref<TrayState>('neutral');
const activeFilter = ref('all');

// Filters
const filters = computed(() => [
  { key: 'all', label: 'All', count: totalOpen.value, variant: 'neutral' as const },
  { key: 'stale', label: 'Stale', count: staleCount.value, variant: 'amber' as const },
  { key: 'review', label: 'To Review', count: pendingReview.value, variant: 'primary' as const },
]);

const filteredPrs = computed(() => {
  switch (activeFilter.value) {
    case 'stale':
      return prs.value.filter(pr => pr.isStale);
    case 'review':
      return prs.value.filter(pr => pr.reviewers.some(r => !r.approved));
    default:
      return prs.value;
  }
});

const statusMessage = computed(() => {
  if (staleCount.value > 0) {
    return `${staleCount.value} stale PR${staleCount.value > 1 ? 's' : ''} need attention`;
  }
  if (pendingReview.value > 0) {
    return `${pendingReview.value} PR${pendingReview.value > 1 ? 's' : ''} awaiting review`;
  }
  return 'All PRs up to date';
});

function setFilter(key: string) {
  activeFilter.value = key;
}

async function loadData() {
  loading.value = true;
  try {
    const [summary, prList] = await Promise.all([
      getPrSummary(),
      getPrs({}),
    ]);
    
    totalOpen.value = summary.totalOpen;
    staleCount.value = summary.staleCount;
    pendingReview.value = summary.pendingReview;
    trayState.value = summary.trayState as TrayState;
    prs.value = prList;
  } catch (error) {
    console.error('Failed to load PR data:', error);
  } finally {
    loading.value = false;
  }
}

async function refresh() {
  loading.value = true;
  try {
    await refreshPrs();
    await loadData();
  } finally {
    loading.value = false;
  }
}

function openPr(pr: PrItem) {
  if (pr.url) {
    window.open(pr.url, '_blank');
  }
}

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.pr-radar {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: var(--space-4);
}

.radar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
}

.header-content {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.radar-title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-xl);
  font-weight: var(--font-semibold);
  margin: 0;
}

.title-icon {
  font-size: 1.25rem;
}

.header-stats {
  display: flex;
  gap: var(--space-6);
}

.stat {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: var(--text-2xl);
  font-weight: var(--font-bold);
  color: var(--text-primary);
}

.stat-label {
  font-size: var(--text-xs);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.stat--warning .stat-value {
  color: var(--color-amber-500);
}

.stat--accent .stat-value {
  color: var(--color-primary-400);
}

.refresh-btn {
  padding: var(--space-2);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.refresh-btn:hover:not(:disabled) {
  background: var(--glass-bg-hover);
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.refresh-icon {
  font-size: 1.25rem;
  display: block;
}

.radar-filters {
  display: flex;
  gap: var(--space-2);
  padding: 0 var(--space-4);
}

.filter-chip {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-full);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.filter-chip:hover {
  background: var(--glass-bg-hover);
  color: var(--text-primary);
}

.filter-chip--active {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.radar-content {
  flex: 1;
  overflow-y: auto;
  padding: 0 var(--space-4);
}

.loading-skeleton {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.skeleton {
  height: 80px;
  border-radius: var(--radius-lg);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: var(--space-4);
}

.empty-subtitle {
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.pr-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

/* Transition animations */
.pr-list-enter-active {
  animation: result-slide-in var(--duration-normal) var(--ease-out) forwards;
}

.pr-list-leave-active {
  animation: result-slide-in var(--duration-fast) var(--ease-in) reverse forwards;
}

.pr-list-move {
  transition: transform var(--duration-normal) var(--ease-out);
}
</style>
