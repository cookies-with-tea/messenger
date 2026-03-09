<template>
  <button
    @click="emit('select')"
    class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 relative"
    :class="active
      ? 'bg-elevated border border-border shadow-glow'
      : 'hover:bg-surface/60 border border-transparent'"
  >
    <!-- Active indicator -->
    <div
      v-if="active"
      class="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-6 bg-pulse rounded-r-full"
    />

    <!-- Avatar -->
    <ChatAvatar :chat="chat.sender" :size="42" />

    <div class="flex-1 min-w-0 text-left">
      <div class="flex items-center justify-between gap-2">
        <span class="text-sm font-semibold truncate" :class="active ? 'text-text-bright' : 'text-text-base'">
          {{ chat.sender.first_name ?? 'Chat' }} {{ chat.sender?.second_name }}
        </span>
        <span class="text-xs font-mono shrink-0" :class="active ? 'text-text-dim' : 'text-muted'">
          {{ timeLabel }}
        </span>
      </div>
      <div class="flex items-center justify-between gap-2 mt-0.5">
        <p class="text-xs truncate" :class="(chat.unread_count ?? 0) > 0 ? 'text-text-base' : 'text-text-dim'">
          {{ chat.last_message_body ?? 'No messages yet' }}
        </p>
        <span
          v-if="(chat.unread_count ?? 0) > 0"
          class="shrink-0 min-w-[18px] h-[18px] px-1 rounded-full bg-pulse text-white text-[10px] font-bold flex items-center justify-center"
        >
          {{ (chat.unread_count ?? 0) > 9 ? '9+' : chat.unread_count }}
        </span>
      </div>
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ChatResponseDTO } from '@/types'
import ChatAvatar from './ChatAvatar.vue'

const props = defineProps<{ chat: ChatResponseDTO; active: boolean }>()
const emit  = defineEmits<{ select: [] }>()

const timeLabel = computed(() => {
  const ts = props.chat.last_message_at
  if (!ts) return ''
  const diff = Date.now() - new Date(ts).getTime()
  if (diff < 60_000)    return 'now'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m`
  if (diff < 86_400_000)return `${Math.floor(diff / 3_600_000)}h`
  return `${Math.floor(diff / 86_400_000)}d`
})
</script>
