<template>
  <div class="relative inline-flex items-center justify-center shrink-0 select-none" :style="sizeStyle">
    <div
      class="flex items-center justify-center rounded-full font-semibold transition-all overflow-hidden"
      :class="containerClass"
      :style="sizeStyle"
    >
      <AppImage
        v-if="props.user?.avatar?.url"
        :src="props.user.avatar.url"
        :alt="props.user.avatar.alt || ''"
        :initials="initials"
        :size="size"
        class-name="w-full h-full"
      />
      <span v-else :style="{ fontSize: `${size * 0.45}px` }">{{ initials }}</span>
    </div>
    <span
      v-if="props.showStatus && props.user"
      class="absolute bottom-0 right-0 rounded-full border-2 border-abyss"
      :class="statusDotClass"
      :style="{ width: `${size * 0.28}px`, height: `${size * 0.28}px` }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AppImage from './ui/AppImage.vue'
import type { UserPreviewDTO } from '@/types'

const props = defineProps<{
  user?: UserPreviewDTO
  size?: number
  showStatus?: boolean
}>()

const size = computed(() => props.size ?? 36)
const sizeStyle = computed(() => ({ width: `${size.value}px`, height: `${size.value}px` }))

const containerClass = computed(() => 'bg-elevated border border-border')

const initials = computed(() => {
  if (!props.user) return '?'
  const first = props.user.first_name?.[0] || ''
  const last = props.user.second_name?.[0] || ''
  return (first + last).toUpperCase() || props.user.uuid.slice(0, 2).toUpperCase()
})

const statusDotClass = computed(() => ({
  'bg-sage': props.user?.is_online,
  'bg-muted': !props.user?.is_online,
}))
</script>
