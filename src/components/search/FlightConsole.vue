<template>
  <Teleport to="body">
    <Transition name="spotlight">
      <div v-if="isOpen" class="spotlight-container" @click.self="close">
        <div class="spotlight-backdrop" @click="close"></div>
        
        <div class="spotlight-panel slide-up" @keydown="handleKeydown">
          <SearchInput
            ref="searchInputRef"
            v-model="query"
            :loading="loading"
            :placeholder="placeholder"
            @search="performSearch"
            @navigate="handleNavigate"
            @close="close"
          />
          
          <SearchResults
            :results="results"
            :selected-index="selectedIndex"
            :loading="loading"
            :query="query"
            @select="handleSelect"
            @hover="selectedIndex = $event"
            @action="handleQuickAction"
          />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import SearchInput from './SearchInput.vue';
import SearchResults from './SearchResults.vue';
import { search as searchApi, isCommandError } from '../../composables/useTauri';
import type { SearchResult } from '../../types';

interface Props {
  modelValue?: boolean;
  placeholder?: string;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  placeholder: 'Search tickets, PRs, docs...',
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'select': [result: SearchResult];
}>();

const isOpen = ref(props.modelValue);
const query = ref('');
const results = ref<SearchResult[]>([]);
const loading = ref(false);
const selectedIndex = ref(0);
const searchInputRef = ref<InstanceType<typeof SearchInput> | null>(null);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// Sync with v-model
watch(() => props.modelValue, (newValue) => {
  isOpen.value = newValue;
  if (newValue) {
    // Reset state when opening
    query.value = '';
    results.value = [];
    selectedIndex.value = 0;
    // Focus input after mount
    setTimeout(() => searchInputRef.value?.focus(), 50);
  }
});

watch(isOpen, (newValue) => {
  emit('update:modelValue', newValue);
});

function close() {
  isOpen.value = false;
}

async function performSearch(searchQuery: string) {
  if (!searchQuery.trim()) {
    results.value = [];
    return;
  }

  // Debounce search
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = setTimeout(async () => {
    loading.value = true;
    try {
      const response = await searchApi({ query: searchQuery });
      results.value = response.results;
      selectedIndex.value = 0;
    } catch (error) {
      console.error('Search failed:', error);
      if (isCommandError(error)) {
        console.error(`Error ${error.code}: ${error.message}`);
      }
      results.value = [];
    } finally {
      loading.value = false;
    }
  }, 200);
}

function handleNavigate(direction: 'up' | 'down') {
  if (results.value.length === 0) return;

  if (direction === 'down') {
    selectedIndex.value = (selectedIndex.value + 1) % results.value.length;
  } else {
    selectedIndex.value = (selectedIndex.value - 1 + results.value.length) % results.value.length;
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && results.value.length > 0) {
    handleSelect(results.value[selectedIndex.value]);
  }
}

function handleSelect(result: SearchResult) {
  emit('select', result);
  
  // Open URL if available
  if (result.url) {
    window.open(result.url, '_blank');
  }
  
  close();
}

function handleQuickAction(action: string) {
  switch (action) {
    case 'my-tickets':
      query.value = 'assignee:me';
      performSearch(query.value);
      break;
    case 'pending-reviews':
      query.value = 'type:pr reviewer:me';
      performSearch(query.value);
      break;
    case 'incidents':
      query.value = 'type:incident';
      performSearch(query.value);
      break;
  }
}

// Global keyboard shortcut
function handleGlobalKeydown(event: KeyboardEvent) {
  // Alt+Space or Cmd+K to open
  if ((event.altKey && event.code === 'Space') || (event.metaKey && event.key === 'k')) {
    event.preventDefault();
    isOpen.value = !isOpen.value;
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
});

// Expose methods for parent
defineExpose({
  open: () => { isOpen.value = true; },
  close,
});
</script>

<style scoped>
/* Spotlight transition */
.spotlight-enter-active,
.spotlight-leave-active {
  transition: opacity var(--transition-normal);
}

.spotlight-enter-active .spotlight-panel,
.spotlight-leave-active .spotlight-panel {
  transition: transform var(--transition-normal), opacity var(--transition-normal);
}

.spotlight-enter-from,
.spotlight-leave-to {
  opacity: 0;
}

.spotlight-enter-from .spotlight-panel,
.spotlight-leave-to .spotlight-panel {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}
</style>
