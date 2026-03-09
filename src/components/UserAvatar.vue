<template>
  <div class="relative inline-flex items-center justify-center shrink-0 select-none" :style="sizeStyle">
    <div
      class="flex items-center justify-center rounded-full font-semibold transition-all"
      :class="containerClass"
      :style="sizeStyle"
    >
      <span :style="{ fontSize: `${size * 0.45}px` }">{{ user?.avatar || '?' }}</span>
    </div>
    <span
      v-if="showStatus && user"
      class="absolute bottom-0 right-0 rounded-full border-2 border-abyss"
      :class="statusDotClass"
      :style="{ width: `${size * 0.28}px`, height: `${size * 0.28}px` }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { User } from '@/types'

const props = defineProps<{
  user?: User
  size?: number
  showStatus?: boolean
}>()

const size = computed(() => props.size ?? 36)
const sizeStyle = computed(() => ({ width: `${size.value}px`, height: `${size.value}px` }))

const containerClass = computed(() => 'bg-elevated border border-border')

const statusDotClass = computed(() => ({
  'bg-sage': props.user?.status === 'online',
  'bg-ember': props.user?.status === 'away',
  'bg-muted': props.user?.status === 'offline',
}))
</script>
