<template>
  <div
    class="flex items-center justify-center rounded-full bg-elevated border border-border shrink-0 select-none"
    :style="{ width: `${size}px`, height: `${size}px`, fontSize: `${size * 0.4}px` }"
  >
    <span v-if="props.chat.avatar">
      <img :src="props.chat.avatar.url" :alt="props.chat.avatar.alt" :title="props.chat.avatar.title" class="rounded-full" :style="{ width: `${size}px`, height: `${size}px` }" />
    </span>
    <span v-else>{{ initials }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ChatResponseDTO } from '@/types'

const props = defineProps<{ chat: any }>()
const size  = computed(() => props.size ?? 36)

const initials = computed(() => {
  const name = props.chat.name ?? ''
  if (!name) return props.chat.chat_type === 'group' ? '👥' : '💬'
  return name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase()
})
</script>
