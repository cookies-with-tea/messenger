<template>
  <div class="flex items-center gap-2.5 animate-fade-in pl-1">
    <div class="flex items-center gap-2 px-3 py-1.5 rounded-2xl rounded-bl-sm glass border border-white/5 backdrop-blur-md">
      <span class="text-[10px] text-text-dim font-mono uppercase tracking-widest">{{ label }}</span>
      <span class="flex gap-1 items-center">
        <span class="w-1 h-1 rounded-full bg-pulse animate-bounce shadow-[0_0_8px_rgba(var(--color-pulse),0.6)]" style="animation-delay: 0ms" />
        <span class="w-1 h-1 rounded-full bg-pulse animate-bounce shadow-[0_0_8px_rgba(var(--color-pulse),0.6)]" style="animation-delay: 150ms" />
        <span class="w-1 h-1 rounded-full bg-pulse animate-bounce shadow-[0_0_8px_rgba(var(--color-pulse),0.6)]" style="animation-delay: 300ms" />
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ userIds: string[] }>()

const label = computed(() => {
  if (props.userIds.length === 1) return 'typing'
  if (props.userIds.length === 2) return '2 typing'
  return 'multiple typing'
})
</script>
