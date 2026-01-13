<template>
  <article 
    :class="['pr-card', 'glass-card', 'hover-lift', { 'stale-glow': pr.isStale }]"
    @click="$emit('click', pr)"
    tabindex="0"
    @keydown.enter="$emit('click', pr)"
  >
    <div class="pr-header">
      <div class="pr-repo">
        <span class="repo-icon">📁</span>
        {{ pr.repository }}
      </div>
      <Badge v-if="pr.isStale" variant="amber">Stale</Badge>
      <Badge :variant="checksVariant">{{ checksLabel }}</Badge>
    </div>

    <h3 class="pr-title">{{ pr.title }}</h3>

    <div class="pr-meta">
      <div class="pr-author">
        <img 
          v-if="pr.author.avatar" 
          :src="pr.author.avatar" 
          :alt="pr.author.name"
          class="author-avatar"
        />
        <span v-else class="author-initial">{{ pr.author.name.charAt(0) }}</span>
        <span class="author-name">{{ pr.author.name }}</span>
      </div>

      <div class="pr-branches">
        <code class="branch">{{ pr.sourceBranch }}</code>
        <span class="branch-arrow">→</span>
        <code class="branch">{{ pr.targetBranch }}</code>
      </div>
    </div>

    <div class="pr-footer">
      <div class="pr-reviewers" v-if="pr.reviewers.length > 0">
        <span class="reviewer-label">Reviewers:</span>
        <div class="reviewer-list">
          <span 
            v-for="reviewer in pr.reviewers.slice(0, 3)" 
            :key="reviewer.user.id"
            :class="['reviewer', { 'reviewer--approved': reviewer.approved }]"
            :title="reviewer.user.name"
          >
            {{ reviewer.approved ? '✅' : '👤' }}
          </span>
          <span v-if="pr.reviewers.length > 3" class="reviewer-more">
            +{{ pr.reviewers.length - 3 }}
          </span>
        </div>
      </div>

      <div class="pr-age">
        <span :class="['age-icon', { 'age-icon--warning': pr.ageHours >= 48 }]">⏱️</span>
        <span class="age-text">{{ formatAge(pr.ageHours) }}</span>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Badge from '../common/Badge.vue';
import type { PrItem } from '../../types';

interface Props {
  pr: PrItem;
}

const props = defineProps<Props>();

defineEmits<{
  'click': [pr: PrItem];
}>();

const checksVariant = computed(() => {
  const status = props.pr.checksStatus.toLowerCase();
  if (status === 'pass' || status === 'success') return 'green';
  if (status === 'fail' || status === 'failure') return 'red';
  if (status === 'pending') return 'amber';
  return 'neutral';
});

const checksLabel = computed(() => {
  const status = props.pr.checksStatus.toLowerCase();
  if (status === 'pass' || status === 'success') return 'Passing';
  if (status === 'fail' || status === 'failure') return 'Failing';
  if (status === 'pending') return 'Pending';
  return props.pr.checksStatus;
});

function formatAge(hours: number): string {
  if (hours < 1) return 'Just now';
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days === 1) return '1 day ago';
  if (days < 7) return `${days} days ago`;
  const weeks = Math.floor(days / 7);
  return weeks === 1 ? '1 week ago' : `${weeks} weeks ago`;
}
</script>

<style scoped>
.pr-card {
  padding: var(--space-4);
  cursor: pointer;
  outline: none;
  transition: 
    background var(--transition-fast),
    transform var(--transition-fast),
    box-shadow var(--transition-fast);
}

.pr-card:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.pr-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.pr-repo {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-sm);
  color: var(--text-muted);
  flex: 1;
}

.repo-icon {
  font-size: 0.875rem;
}

.pr-title {
  font-size: var(--text-base);
  font-weight: var(--font-medium);
  color: var(--text-primary);
  margin: 0 0 var(--space-3) 0;
  line-height: var(--leading-tight);
}

.pr-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-3);
}

.pr-author {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.author-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  object-fit: cover;
}

.author-initial {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--glass-bg);
  border-radius: 50%;
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  color: var(--text-secondary);
}

.author-name {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.pr-branches {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.branch {
  font-size: var(--text-xs);
  padding: var(--space-1) var(--space-2);
  background: var(--glass-bg);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.branch-arrow {
  color: var(--text-muted);
  font-size: var(--text-sm);
}

.pr-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: var(--space-3);
  border-top: 1px solid var(--glass-border);
}

.pr-reviewers {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.reviewer-label {
  font-size: var(--text-xs);
  color: var(--text-muted);
}

.reviewer-list {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.reviewer {
  font-size: var(--text-sm);
  opacity: 0.6;
}

.reviewer--approved {
  opacity: 1;
}

.reviewer-more {
  font-size: var(--text-xs);
  color: var(--text-muted);
}

.pr-age {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.age-icon {
  font-size: var(--text-sm);
}

.age-icon--warning {
  animation: status-pulse 2s ease-in-out infinite;
}

.age-text {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

/* Stale glow animation from animations.css */
.stale-glow {
  border: 1px solid var(--color-amber-500);
}
</style>
