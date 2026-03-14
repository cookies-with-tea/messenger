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
    <ChatAvatar :chat="chat.sender" :size="42" :show-status="chat.chat_type === 'direct'" />

    <div class="flex-1 min-w-0 text-left relative overflow-hidden">
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-center gap-1.5 min-w-0">
          <svg v-if="messenger.pinnedChatUuids.has(chat.uuid)" class="w-3 h-3 text-pulse shrink-0 rotate-45" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
          </svg>
          <span class="text-sm font-semibold truncate" :class="active ? 'text-text-bright' : 'text-text-base'">
            {{ displayName }}
          </span>
        </div>
        <span class="text-xs font-mono shrink-0" :class="active ? 'text-text-dim' : 'text-muted'">
          {{ timeLabel }}
        </span>
      </div>
      <div class="flex items-center justify-between gap-2 mt-0.5">
        <p class="text-xs truncate flex-1" :class="(chat.unread_count ?? 0) > 0 ? 'text-text-base' : 'text-text-dim'">
          {{ chat.last_message_body ?? 'No messages yet' }}
        </p>
        
        <!-- Hover Actions -->
        <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-2">
          <button 
            @click.stop="messenger.togglePinChat(chat.uuid)"
            class="p-1 rounded hover:bg-white/10 text-muted hover:text-pulse transition-colors"
            :title="messenger.pinnedChatUuids.has(chat.uuid) ? 'Unpin' : 'Pin'"
          >
            <svg class="w-3 h-3" :class="messenger.pinnedChatUuids.has(chat.uuid) ? 'rotate-45 text-pulse' : ''" viewBox="0 0 20 20" fill="currentColor">
              <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
            </svg>
          </button>
          <button 
            @click.stop="messenger.toggleArchiveChat(chat.uuid)"
            class="p-1 rounded hover:bg-white/10 text-muted hover:text-ember transition-colors"
            :title="messenger.archivedChatUuids.has(chat.uuid) ? 'Unarchive' : 'Archive'"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor">
              <path d="M4 3a2 2 0 100 4h12a2 2 0 100-4H4z" />
              <path fill-rule="evenodd" d="M3 8h14v7a2 2 0 01-2 2H5a2 2 0 01-2-2V8zm5 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>

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
import { useMessengerStore } from '@/stores/messengerStore'
import ChatAvatar from './ChatAvatar.vue'

const props = defineProps<{ chat: ChatResponseDTO; active: boolean }>()
const emit  = defineEmits<{ select: [] }>()

const messenger = useMessengerStore()

const displayName = computed(() => {
  if (props.chat.alias) return props.chat.alias
  const sender = props.chat.sender
  return sender ? `${sender.first_name || 'Chat'} ${sender.second_name || ''}`.trim() : (props.chat.name || 'Chat')
})

const timeLabel = computed(() => {
  const ts = props.chat.last_message_at
  if (!ts) return ''
  const date = new Date(ts)
  if (isNaN(date.getTime()) || date.getFullYear() <= 1970) return ''

  const diff = messenger.now.getTime() - date.getTime()
  if (diff < 60_000) return 'сейчас'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} мин.`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} ч.`
  
  const day = date.getDate().toString().padStart(2, '0')
  const month = (date.getMonth() + 1).toString().padStart(2, '0')
  const year = date.getFullYear()
  return `${day}.${month}.${year}`
})
</script>
