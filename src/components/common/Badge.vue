<template>
  <span :class="['badge', `badge--${variant}`]">
    <slot></slot>
  </span>
</template>

<script setup lang="ts">
type Variant = 'primary' | 'green' | 'amber' | 'red' | 'neutral';

interface Props {
  variant?: Variant;
}

withDefaults(defineProps<Props>(), {
  variant: 'neutral',
});
</script>

<style scoped>
.badge {
  display: inline-flex;
  align-items: center;
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  border-radius: var(--radius-full);
  white-space: nowrap;
}

.badge--neutral {
  background: var(--glass-bg);
  color: var(--text-secondary);
}

.badge--primary {
  background: var(--color-primary-500);
  color: white;
}

.badge--green {
  background: rgba(34, 197, 94, 0.2);
  color: #22c55e;
}

.badge--amber {
  background: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
}

.badge--red {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}
</style>
