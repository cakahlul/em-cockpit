<template>
  <div id="app" :data-theme="theme">
    <!-- Main App Content -->
    <main class="app-main">
      <div class="app-header">
        <h1 class="app-title">
          <span class="title-icon">🛫</span>
          EM Cockpit
        </h1>
        <div class="app-status">
          <StatusDot :state="systemState" />
          <span class="status-text">{{ statusText }}</span>
        </div>
      </div>

      <div class="app-content">
        <div class="welcome-panel glass-panel">
          <h2>Welcome, Commander</h2>
          <p>Press <kbd>Alt+Space</kbd> to open Flight Console</p>
          
          <div class="quick-stats">
            <div class="stat-card glass-card">
              <div class="stat-icon">🔀</div>
              <div class="stat-value">{{ prCount }}</div>
              <div class="stat-label">Open PRs</div>
            </div>
            <div class="stat-card glass-card">
              <div class="stat-icon">🎫</div>
              <div class="stat-value">{{ ticketCount }}</div>
              <div class="stat-label">My Tickets</div>
            </div>
            <div class="stat-card glass-card">
              <div class="stat-icon">🚨</div>
              <div class="stat-value">{{ incidentCount }}</div>
              <div class="stat-label">Incidents</div>
            </div>
          </div>
        </div>

        <div class="shortcut-guide glass-panel">
          <h3>Keyboard Shortcuts</h3>
          <div class="shortcuts-list">
            <div class="shortcut-item">
              <kbd>Alt+Space</kbd>
              <span>Flight Console (Search)</span>
            </div>
            <div class="shortcut-item">
              <kbd>Ctrl+2</kbd>
              <span>Radar Panel (PRs)</span>
            </div>
            <div class="shortcut-item">
              <kbd>Ctrl+3</kbd>
              <span>Incident Radar</span>
            </div>
            <div class="shortcut-item">
              <kbd>Esc</kbd>
              <span>Close Panel</span>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- Flight Console (Spotlight Search) -->
    <FlightConsole
      v-model="showFlightConsole"
      @select="handleSearchSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import FlightConsole from './components/search/FlightConsole.vue';
import StatusDot from './components/common/StatusDot.vue';
import type { SearchResult, TrayState } from './types';
import { getPrSummary, getIncidentSummary } from './composables/useTauri';

// App state
const showFlightConsole = ref(false);
const theme = ref<'light' | 'dark'>('dark');

// Stats
const prCount = ref(0);
const ticketCount = ref(0);
const incidentCount = ref(0);
const systemState = ref<TrayState>('neutral');

const statusText = computed(() => {
  switch (systemState.value) {
    case 'red': return 'Critical incidents active';
    case 'amber': return 'Attention needed';
    case 'green': return 'All systems nominal';
    default: return 'Monitoring...';
  }
});

function handleSearchSelect(result: SearchResult) {
  console.log('Selected:', result);
}

async function loadStats() {
  try {
    const prSummary = await getPrSummary();
    prCount.value = prSummary.totalOpen;
    
    const incidentSummary = await getIncidentSummary();
    incidentCount.value = incidentSummary.totalActive;
    
    // Determine system state
    if (incidentSummary.criticalCount > 0 || incidentSummary.highCount > 0) {
      systemState.value = 'red';
    } else if (prSummary.staleCount > 0 || incidentSummary.mediumCount > 0) {
      systemState.value = 'amber';
    } else if (prSummary.totalOpen === 0 && incidentSummary.totalActive === 0) {
      systemState.value = 'green';
    } else {
      systemState.value = 'neutral';
    }
  } catch (error) {
    console.error('Failed to load stats:', error);
  }
}

onMounted(() => {
  loadStats();
  // Refresh stats every 2 minutes
  setInterval(loadStats, 120000);
});
</script>

<style>
@import './assets/design-system.css';

#app {
  min-height: 100vh;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%);
}

.app-main {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8);
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-8);
}

.app-title {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-3xl);
  font-weight: var(--font-bold);
  color: var(--text-primary);
}

.title-icon {
  font-size: 2rem;
}

.app-status {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-full);
}

.status-text {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.app-content {
  display: grid;
  gap: var(--space-6);
}

.welcome-panel {
  padding: var(--space-8);
  text-align: center;
}

.welcome-panel h2 {
  font-size: var(--text-2xl);
  margin-bottom: var(--space-4);
}

.welcome-panel p {
  color: var(--text-secondary);
  margin-bottom: var(--space-8);
}

.welcome-panel kbd {
  padding: var(--space-1) var(--space-2);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
}

.quick-stats {
  display: flex;
  gap: var(--space-4);
  justify-content: center;
}

.stat-card {
  padding: var(--space-6);
  min-width: 140px;
  text-align: center;
}

.stat-icon {
  font-size: 2rem;
  margin-bottom: var(--space-2);
}

.stat-value {
  font-size: var(--text-3xl);
  font-weight: var(--font-bold);
  color: var(--text-primary);
}

.stat-label {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin-top: var(--space-1);
}

.shortcut-guide {
  padding: var(--space-6);
}

.shortcut-guide h3 {
  font-size: var(--text-lg);
  margin-bottom: var(--space-4);
}

.shortcuts-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--space-3);
}

.shortcut-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2);
}

.shortcut-item kbd {
  padding: var(--space-1) var(--space-3);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  min-width: 90px;
  text-align: center;
}

.shortcut-item span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}
</style>