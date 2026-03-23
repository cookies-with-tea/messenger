<template>
  <div class="relative inline-flex shrink-0">
    <div
      class="flex items-center justify-center rounded-full bg-elevated border border-border shrink-0 select-none overflow-hidden"
      :style="{ width: `${size}px`, height: `${size}px`, fontSize: `${size * 0.4}px` }"
    >
      <AppImage
        v-if="avatarUrl"
        :src="avatarUrl"
        :alt="avatarAlt"
        :initials="initials"
        :size="size"
        class-name="w-full h-full"
      />
      <span v-else class="font-semibold">{{ initials }}</span>
    </div>
    
    <!-- Online status dot -->
    <span
      v-if="props.showStatus && isOnline"
      class="absolute bottom-0 right-0 rounded-full border-2 border-abyss bg-sage"
      :style="{ width: `${size * 0.28}px`, height: `${size * 0.28}px` }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AppImage from './ui/AppImage.vue'
// import type { ChatResponseDTO, UserPreviewDTO } from '@/types'

const props = defineProps<{ 
  chat?: any // Can be ChatResponseDTO or UserPreviewDTO
  size?: number
  showStatus?: boolean
}>()

const size = computed(() => props.size ?? 36)

const isUser = computed(() => !!props.chat?.uuid && ('first_name' in props.chat))

const avatarUrl = computed(() => {
  if (typeof props.chat?.avatar === 'string') return props.chat.avatar
  return props.chat?.avatar?.url
})

const avatarAlt = computed(() => props.chat?.avatar?.alt || '')
// const avatarTitle = computed(() => props.chat?.avatar?.title || '')

const isOnline = computed(() => props.chat?.is_online ?? false)

const initials = computed(() => {
  if (isUser.value) {
    const first = props.chat.first_name?.[0] || ''
    const second = props.chat.second_name?.[0] || ''
    return (first + second).toUpperCase() || props.chat.uuid.slice(0, 2).toUpperCase()
  }
  
  const name = props.chat?.name ?? ''
  if (!name) return props.chat?.chat_type === 'group' ? '👥' : '💬'
  return name.split(' ').map((w: string) => w[0]).join('').slice(0, 2).toUpperCase()
})
</script>
