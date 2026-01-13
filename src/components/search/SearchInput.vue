<template>
  <div class="search-input-wrapper">
    <div class="search-icon">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
    </div>
    
    <input
      ref="inputRef"
      v-model="query"
      type="text"
      class="search-input"
      :placeholder="placeholder"
      @input="handleInput"
      @keydown.enter="handleSubmit"
      @keydown.down.prevent="$emit('navigate', 'down')"
      @keydown.up.prevent="$emit('navigate', 'up')"
      @keydown.escape="$emit('close')"
    />
    
    <div v-if="loading" class="search-loader">
      <div class="spinner"></div>
    </div>
    
    <div v-else-if="query" class="search-clear" @click="clearQuery">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6 6 18M6 6l12 12" />
      </svg>
    </div>

    <div class="search-shortcut">
      <kbd>{{ shortcut }}</kbd>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

interface Props {
  modelValue: string;
  placeholder?: string;
  loading?: boolean;
  shortcut?: string;
  autofocus?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Search tickets, PRs, docs...',
  loading: false,
  shortcut: 'Alt+Space',
  autofocus: true,
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'search': [query: string];
  'navigate': [direction: 'up' | 'down'];
  'close': [];
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const query = ref(props.modelValue);

watch(() => props.modelValue, (newValue) => {
  query.value = newValue;
});

watch(query, (newValue) => {
  emit('update:modelValue', newValue);
});

function handleInput() {
  emit('search', query.value);
}

function handleSubmit() {
  emit('search', query.value);
}

function clearQuery() {
  query.value = '';
  emit('update:modelValue', '');
  inputRef.value?.focus();
}

onMounted(() => {
  if (props.autofocus) {
    inputRef.value?.focus();
  }
});

defineExpose({
  focus: () => inputRef.value?.focus(),
});
</script>

<style scoped>
.search-input-wrapper {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--glass-border);
}

.search-icon {
  color: var(--text-muted);
  display: flex;
  align-items: center;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: var(--text-lg);
  font-family: var(--font-family);
  color: var(--text-primary);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-loader {
  display: flex;
  align-items: center;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--glass-border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.search-clear {
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  padding: var(--space-1);
  border-radius: var(--radius-sm);
  transition: color var(--transition-fast), background var(--transition-fast);
}

.search-clear:hover {
  color: var(--text-primary);
  background: var(--glass-bg-hover);
}

.search-shortcut {
  display: flex;
  gap: var(--space-1);
}

.search-shortcut kbd {
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  font-family: var(--font-family);
  color: var(--text-muted);
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-sm);
}
</style>
