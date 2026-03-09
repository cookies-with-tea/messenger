<template>
  <div class="flex items-center gap-2.5 animate-fade-in">
    <div class="flex items-center gap-1 px-3 py-2 rounded-2xl rounded-bl-sm bg-elevated border border-border">
      <span class="text-xs text-text-dim font-mono mr-1">{{ label }}</span>
      <span class="flex gap-0.5">
        <span class="w-1.5 h-1.5 rounded-full bg-text-dim animate-bounce" style="animation-delay: 0ms" />
        <span class="w-1.5 h-1.5 rounded-full bg-text-dim animate-bounce" style="animation-delay: 150ms" />
        <span class="w-1.5 h-1.5 rounded-full bg-text-dim animate-bounce" style="animation-delay: 300ms" />
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ userIds: string[] }>()

const label = computed(() => {
  if (props.userIds.length === 1) return 'typing...'
  if (props.userIds.length === 2) return '2 people typing...'
  return 'Several people typing...'
})
</script>
