<template>
  <div
    class="flex gap-2.5 group animate-slide-up"
    :class="isOwn ? 'flex-row-reverse' : 'flex-row'"
  >
    <!-- Avatar placeholder for alignment -->
    <div class="w-8 shrink-0">
      <div
        v-if="!isOwn && showAvatar"
        class="w-8 h-8 rounded-xl overflow-hidden glass border border-white/10 flex items-center justify-center transition-transform group-hover:scale-105"
      >
      	<ChatAvatar :chat="message.sender" :size="32" />
      </div>
    </div>

    <div class="flex flex-col gap-1 max-w-[75%] sm:max-w-[70%]" :class="isOwn ? 'items-end' : 'items-start'">
      <!-- Sender name in group -->
      <span
        v-if="isGroup && !isOwn && showAvatar"
        class="text-[10px] font-mono font-black uppercase tracking-widest px-1 py-0.5"
        :style="{ color: senderColor }"
      >
        {{ senderDisplayName }}
      </span>

      <!-- Reply preview -->
      <div
        v-if="message.reply_body_preview"
        class="text-[10px] font-mono px-3 py-1.5 rounded-xl border border-white/5 text-text-dim bg-white/2 max-w-full truncate backdrop-blur-sm mb-0.5"
      >
        <span class="opacity-50 italic">Replying to:</span> {{ message.reply_body_preview }}
      </div>

      <!-- Bubble -->
      <div
        class="relative rounded-2xl transition-all duration-300"
        :class="[bubbleClass, message.media?.media_type === 'audio' ? 'px-2 py-1' : 'px-4 py-2.5']"
      >
        <VoiceMessage 
          v-if="message.media?.media_type === 'audio'" 
          :src="message.media.url"
          :is-own="isOwn"
        />
        <p v-else class="text-xs sm:text-sm leading-relaxed break-words whitespace-pre-wrap font-mono" :class="textClass">
          {{ message.is_deleted ? 'Signal lost' : message.body }}
        </p>
        <span v-if="message.is_edited && !message.is_deleted" class="absolute -bottom-1 -right-1 text-[8px] font-mono bg-void/80 px-1 rounded border border-white/5 text-muted uppercase">edited</span>
      </div>

      <!-- Meta -->
      <div class="flex items-center gap-2 px-1 py-0.5" :class="isOwn ? 'flex-row-reverse' : 'flex-row'">
        <span class="text-[9px] font-mono text-muted tracking-tighter">{{ timeLabel }}</span>
        <div v-if="isOwn" class="flex" :class="statusClass">
          <!-- Read: Double check -->
          <svg v-if="displayStatus === 'read'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
            <polyline points="22 10 13 19 9 15" />
          </svg>
          <!-- Delivered: Double check -->
          <svg v-else-if="displayStatus === 'delivered'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
            <polyline points="22 10 13 19 9 15" />
          </svg>
          <!-- Sent: Single check -->
          <svg v-else class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
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
import VoiceMessage from './VoiceMessage.vue'

const COLOR_PALETTE = ['#4f7cff','#00e5ff','#52e07c','#ff6b35','#c084fc','#fb923c','#38bdf8','#f472b6']

const props = defineProps<{
  message: MessageResponseDTO
  showAvatar?: boolean
  isGroup?: boolean
}>()

const store = useMessengerStore()
const isOwn = computed(() => props.message.sender_uuid === store.currentUserId)

const senderColor = computed(() => {
  const uuid = props.message.sender_uuid || ''
  const idx = uuid.length > 0 ? uuid.charCodeAt(0) % COLOR_PALETTE.length : 0
  return COLOR_PALETTE[idx]
})

const senderDisplayName = computed(() => {
  const s = props.message.sender
  if (!s) return props.message.sender_uuid?.split('-')[0] || 'Unknown'
  return `${s.first_name || ''} ${s.second_name || ''}`.trim() || props.message.sender_uuid?.split('-')[0]
})

const bubbleClass = computed(() =>
  isOwn.value
    ? 'bg-pulse text-white rounded-tr-sm shadow-[0_0_20px_rgba(var(--color-pulse),0.2)]'
    : 'glass border border-white/10 text-text-bright rounded-tl-sm backdrop-blur-xl'
)
const textClass = computed(() => isOwn.value ? 'text-white' : 'text-text-bright')

const timeLabel = computed(() => {
  const d = new Date(props.message.created_at)
  return d.toLocaleTimeString('en', { hour: '2-digit', minute: '2-digit', hour12: false })
})

const displayStatus = computed(() => {
  const reads = props.message.read_count || 0
  const dels = props.message.delivered_count || 0
  if (reads > 0) return 'read'
  if (dels > 0) return 'delivered'
  return 'sent'
})

const statusClass = computed(() => ({
  'text-white/30': displayStatus.value !== 'read',
  'text-sage shadow-[0_0_8px_rgba(var(--color-sage),0.4)]':  displayStatus.value === 'read',
}))
</script>
