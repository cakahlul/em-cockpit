<template>
  <span :class="['status-dot', `status-dot--${state}`]" :title="title"></span>
</template>

<script setup lang="ts">
import type { TrayState } from '../../types';

interface Props {
  state: TrayState;
  title?: string;
}

withDefaults(defineProps<Props>(), {
  state: 'neutral',
  title: '',
});
</script>

<style scoped>
.status-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

.status-dot--neutral {
  background: var(--tray-neutral);
}

.status-dot--green {
  background: var(--tray-green);
}

.status-dot--amber {
  background: var(--tray-amber);
}

.status-dot--red {
  background: var(--tray-red);
  animation-duration: 1s;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.7; transform: scale(1.1); }
}
</style>
