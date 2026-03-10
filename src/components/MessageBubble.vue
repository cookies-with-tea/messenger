<template>
  <div
    class="flex gap-2.5 group animate-slide-up"
    :class="isOwn ? 'flex-row-reverse' : 'flex-row'"
  >
    <!-- Avatar placeholder for alignment -->
    <div class="w-7 shrink-0">
      <div
        v-if="!isOwn && showAvatar"
        class="w-7 h-7 rounded-full bg-elevated border border-border flex items-center justify-center text-xs select-none"
        :style="{ color: senderColor }"
      >
      	<ChatAvatar :chat="message.sender" :size="28" />
      </div>
    </div>

    <div class="flex flex-col gap-1 max-w-[70%]" :class="isOwn ? 'items-end' : 'items-start'">
      <!-- Sender name in group -->
      <span
        v-if="isGroup && !isOwn && showAvatar"
        class="text-xs font-semibold px-1 font-mono"
        :style="{ color: senderColor }"
      >
        {{ senderDisplayName }}
      </span>

      <!-- Reply preview -->
      <div
        v-if="message.reply_body_preview"
        class="text-xs font-mono px-2 py-1 rounded-lg border-l-2 border-pulse text-text-dim bg-surface/50 max-w-full truncate"
      >
        {{ message.reply_body_preview }}
      </div>

      <!-- Bubble -->
      <div
        class="relative px-3.5 py-2 rounded-2xl transition-all duration-200"
        :class="[bubbleClass, { 'shadow-glow': isOwn }]"
      >
        <p class="text-sm leading-relaxed break-words whitespace-pre-wrap" :class="textClass">
          {{ message.is_deleted ? 'Message deleted' : message.body }}
        </p>
        <span v-if="message.is_edited && !message.is_deleted" class="text-[10px] text-white/50 ml-1">edited</span>
      </div>

      <!-- Meta -->
      <div class="flex items-center gap-1.5 px-1" :class="isOwn ? 'flex-row-reverse' : 'flex-row'">
        <span class="text-[10px] font-mono text-muted">{{ timeLabel }}</span>
        <div v-if="isOwn" class="flex" :class="statusClass">
          <!-- Read: Double check, blue -->
          <svg v-if="displayStatus === 'read'" class="w-2.5 h-2.5" viewBox="0 0 16 16" fill="currentColor">
            <path d="M13.854 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L6.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
            <path d="M10.354 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L2.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
          </svg>
          <!-- Delivered: Double check, gray -->
          <svg v-else-if="displayStatus === 'delivered'" class="w-2.5 h-2.5" viewBox="0 0 16 16" fill="currentColor">
            <path d="M13.854 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L6.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
            <path d="M10.354 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L2.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
          </svg>
          <!-- Sent: Single check, gray -->
          <svg v-else class="w-2.5 h-2.5" viewBox="0 0 16 16" fill="currentColor">
            <path d="M13.854 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L6.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import type { MessageResponseDTO } from '@/types'
import ChatAvatar from './ChatAvatar.vue'

const COLOR_PALETTE = ['#4f7cff','#00e5ff','#52e07c','#ff6b35','#c084fc','#fb923c','#38bdf8','#f472b6']

const props = defineProps<{
  message: MessageResponseDTO
  showAvatar?: boolean
  isGroup?: boolean
}>()

const store = useMessengerStore()
const isOwn = computed(() => props.message.sender_uuid === store.currentUserId)

const senderColor = computed(() => {
  const idx = props.message.sender_uuid.charCodeAt(0) % COLOR_PALETTE.length
  return COLOR_PALETTE[idx]
})

const senderDisplayName = computed(() => {
  if (!props.message.sender) return props.message.sender_uuid
  return [props.message.sender.first_name, props.message.sender.second_name].filter(Boolean).join(' ') || props.message.sender_uuid
})

const bubbleClass = computed(() =>
  isOwn.value
    ? 'bg-pulse text-white rounded-br-sm'
    : 'bg-elevated border border-border text-text-base rounded-bl-sm'
)
const textClass = computed(() => isOwn.value ? 'text-white' : 'text-text-base')

const timeLabel = computed(() => {
  const d = new Date(props.message.created_at)
  return d.toLocaleTimeString('en', { hour: '2-digit', minute: '2-digit', hour12: false })
})

const displayStatus = computed(() => {
  if (props.message.read_count && props.message.read_count > 0) return 'read'
  if (props.message.delivered_count && props.message.delivered_count > 0) return 'delivered'
  return 'sent'
})

const statusClass = computed(() => ({
  'text-white/50': displayStatus.value !== 'read',
  'text-sage':  displayStatus.value === 'read',
}))
</script>
