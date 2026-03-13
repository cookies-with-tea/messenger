<script setup lang="ts">
import { computed, ref } from 'vue'
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'
import { useMessengerStore } from '@/stores/messengerStore'
import type { MessageResponseDTO } from '@/types'
import ChatAvatar from './ChatAvatar.vue'
import VoiceMessage from './VoiceMessage.vue'
import ContextMenu from './ui/ContextMenu.vue'
import ReadReceiptsModal from './ReadReceiptsModal.vue'

const isReceiptsModalOpen = ref(false)

const md: MarkdownIt = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true
})

md.options.highlight = (str: string, lang: string): string => {
  if (lang && hljs.getLanguage(lang)) {
    try {
      return '<pre><code class="hljs">' +
        hljs.highlight(str, { language: lang, ignoreIllegals: true }).value +
        '</code></pre>';
    } catch (__) { }
  }
  return '<pre><code class="hljs">' + md.utils.escapeHtml(str) + '</code></pre>';
}

const COLOR_PALETTE = ['#4f7cff','#00e5ff','#52e07c','#ff6b35','#c084fc','#fb923c','#38bdf8','#f472b6']

const props = defineProps<{
  message: MessageResponseDTO
  showAvatar?: boolean
  isGroup?: boolean
}>()

const emit = defineEmits(['image-click'])

const store = useMessengerStore()
const isOwn = computed(() => props.message.sender_uuid === store.currentUserId)

const showMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)

const handleContextMenu = (e: MouseEvent) => {
  if (props.message.is_deleted) return
  menuX.value = e.clientX
  menuY.value = e.clientY
  showMenu.value = true
}

const scrollToReply = () => {
  if (props.message.reply_to_uuid) {
    const el = document.getElementById(`msg-${props.message.reply_to_uuid}`)
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}

const toggleReaction = (emoji: string) => {
  store.sendReaction(props.message.uuid, emoji)
}

const QUICK_REACTIONS = ['👍', '❤️', '🔥', '😂', '😮', '😢']

const menuItems = computed(() => {
  const items: any[] = [
    { 
      label: 'Reply', 
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l5 5m-5-5l5-5"/></svg>', 
      action: () => { store.replyingToMessage = props.message } 
    },
    {
      label: props.message.is_pinned ? 'Unpin' : 'Pin',
      icon: props.message.is_pinned 
        ? '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"/></svg>' 
        : '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5h14M12 5v14m-7 0h14"/></svg>',
      action: () => { store.togglePinMessage(props.message.uuid, !props.message.is_pinned) }
    }
  ]

  if (isOwn.value) {
    if (props.message.media?.media_type !== 'audio') {
      items.push({ 
        label: 'Edit', 
        icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/></svg>', 
        action: () => { store.editingMessage = props.message } 
      })
    }
    items.push({ 
      label: 'Delete', 
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>', 
      variant: 'danger',
      action: () => { store.deleteMessage(props.message.uuid) } 
    })
  }

  return items
})

const senderColor = computed(() => {
  const uuid = props.message.sender_uuid || ''
  const idx = uuid.length > 0 ? uuid.charCodeAt(0) % COLOR_PALETTE.length : 0
  return COLOR_PALETTE[idx]
})

const senderDisplayName = computed(() => {
  const s = props.message.sender
  if (!s) return props.message.sender_uuid?.split('-')[0] || 'Unknown'
  return `${s?.first_name || ''} ${s?.second_name || ''}`.trim() || props.message.sender_uuid?.split('-')[0]
})

const bubbleClass = computed(() =>
  isOwn.value
    ? 'bg-pulse text-white rounded-tr-sm shadow-[0_0_20px_rgba(var(--color-pulse),0.2)]'
    : 'glass border border-white/10 text-text-bright rounded-tl-sm backdrop-blur-xl'
)
const textClass = computed(() => isOwn.value ? 'text-white' : 'text-text-bright')

const renderedBody = computed(() => {
  if (props.message.is_deleted) return 'Signal lost'
  return md.render(props.message.body)
})

const reactionsGrouped = computed(() => {
  const rs = props.message.reactions || []
  const groups: Record<string, { count: number, me: boolean }> = {}
  
  rs.forEach(r => {
    if (!groups[r.emoji]) {
      groups[r.emoji] = { count: 0, me: false }
    }
    const group = groups[r.emoji]
    if (group) {
      group.count++
      if (r.user_uuid === store.currentUserId) group.me = true
    }
  })
  
  return Object.entries(groups).map(([emoji, data]) => ({ emoji, ...data }))
})

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

<template>
  <div
    :id="`msg-${message.uuid}`"
    class="flex gap-2.5 group animate-slide-up"
    :class="isOwn ? 'flex-row-reverse' : 'flex-row'"
    @contextmenu.prevent="handleContextMenu"
  >
    <div class="w-8 shrink-0">
      <div
        v-if="!isOwn && showAvatar"
        class="w-8 h-8 rounded-xl overflow-hidden glass border border-white/10 flex items-center justify-center transition-transform group-hover:scale-105"
      >
      	<ChatAvatar 
          :chat="message.sender" 
          :size="32" 
          class="cursor-pointer hover:scale-110 transition-transform"
          @click="store.openProfile(message.sender_uuid)"
        />
      </div>
    </div>

    <div class="flex flex-col gap-1 max-w-[75%] sm:max-w-[70%]" :class="isOwn ? 'items-end' : 'items-start'">
      <span
        v-if="isGroup && !isOwn && showAvatar"
        class="text-[10px] font-mono font-black uppercase tracking-widest px-1 py-0.5"
        :style="{ color: senderColor }"
      >
        {{ senderDisplayName }}
      </span>

      <div
        v-if="message.reply_body_preview"
        class="text-[10px] font-mono px-3 py-1.5 rounded-xl border border-white/5 text-text-dim bg-white/2 max-w-full truncate backdrop-blur-sm mb-0.5 cursor-pointer hover:bg-white/5 transition-colors"
        @click="scrollToReply"
      >
        <span class="opacity-50 italic">Replying to:</span> {{ message.reply_body_preview }}
      </div>

      <div
        class="relative rounded-2xl transition-all duration-300"
        :class="[bubbleClass, message.media?.media_type === 'audio' ? 'px-2 py-1' : 'px-4 py-2.5']"
      >
        <VoiceMessage 
          v-if="message.media?.media_type === 'audio'" 
          :uuid="message.media.uuid"
          :src="message.media.url"
          :title="message.media.title"
          :is-own="isOwn"
        />
        
        <div v-else-if="message.media?.media_type === 'image'" 
             class="mb-1 rounded-lg overflow-hidden glass-heavy border border-white/10 cursor-zoom-in group/img relative"
             @click="emit('image-click', message.media.url)"
        >
          <img :src="message.media.url" class="max-w-full max-h-[300px] object-contain block transition-transform group-hover/img:scale-[1.02]" :alt="message.body" />
          <div class="absolute inset-0 bg-void/0 group-hover/img:bg-void/10 transition-colors flex items-center justify-center">
             <svg class="w-8 h-8 text-white opacity-0 group-hover/img:opacity-50 transition-opacity" viewBox="0 0 20 20" fill="currentColor">
                <path d="M5 8a1 1 0 011-1h1V6a1 1 0 012 0v1h1a1 1 0 110 2H9v1a1 1 0 11-2 0V9H6a1 1 0 01-1-1z" />
                <path fill-rule="evenodd" d="M2 10a8 8 0 1116 0 8 8 0 01-16 0zm8-6a6 6 0 100 12 6 6 0 000-12z" clip-rule="evenodd" />
             </svg>
          </div>
        </div>

        <div v-else-if="message.media" class="flex items-center gap-3 mb-1 p-2 rounded-lg bg-white/5 border border-white/10">
          <svg class="w-6 h-6 text-blue-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
            <polyline points="13 2 13 9 20 9" />
          </svg>
          <div class="flex flex-col overflow-hidden">
            <span class="text-[10px] font-mono truncate text-text-bright">{{ message.media.title || 'Attached File' }}</span>
            <a :href="message.media.url" target="_blank" class="text-[9px] font-mono text-blue-400 hover:underline">Download</a>
          </div>
        </div>

        <div 
          v-if="!message.media || message.media.media_type !== 'audio'" 
          class="text-xs sm:text-sm leading-relaxed wrap-break-word whitespace-pre-wrap font-mono prose prose-invert prose-xs max-w-none" 
          :class="textClass"
          v-html="renderedBody"
        ></div>
        
        <div v-if="message.is_pinned" class="absolute -top-2 -right-2 bg-ember p-1 rounded-full shadow-[0_0_10px_rgba(var(--color-ember),0.6)] z-10">
          <svg class="w-2.5 h-2.5 text-void" viewBox="0 0 20 20" fill="currentColor">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
        </div>

        <!-- Quick Reaction Bar on Hover -->
        <div 
          class="absolute -top-8 left-1/2 -translate-x-1/2 flex gap-1 px-1.5 py-1 glass-heavy rounded-full border border-white/10 shadow-xl scale-90 opacity-0 group-hover:scale-100 group-hover:opacity-100 transition-all duration-200 pointer-events-none group-hover:pointer-events-auto z-30"
          v-if="!message.is_deleted"
        >
          <button 
            v-for="emoji in QUICK_REACTIONS" 
            :key="emoji"
            @click.stop="toggleReaction(emoji)"
            class="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded-full transition-colors text-sm"
          >
            {{ emoji }}
          </button>
        </div>

        <span v-if="message.is_edited && !message.is_deleted" class="absolute -bottom-1 -right-1 text-[8px] font-mono bg-void/80 px-1 rounded border border-white/5 text-muted uppercase">edited</span>

        <div 
          v-if="reactionsGrouped.length" 
          class="absolute -bottom-3 flex flex-wrap gap-1 z-20"
          :class="isOwn ? 'right-0' : 'left-0'"
        >
          <button
            v-for="reg in reactionsGrouped"
            :key="reg.emoji"
            @click="toggleReaction(reg.emoji)"
            class="flex items-center gap-1.5 px-1.5 py-0.5 rounded-full glass border transition-all text-[10px]"
            :class="reg.me ? 'bg-pulse/20 border-pulse/40 text-text-bright' : 'bg-white/5 border-white/10 text-text-dim hover:bg-white/10'"
          >
            <span>{{ reg.emoji }}</span>
            <span v-if="reg.count > 1" class="font-bold opacity-80">{{ reg.count }}</span>
          </button>
        </div>
      </div>

      <div class="flex items-center gap-2 px-1 py-0.5" :class="isOwn ? 'flex-row-reverse' : 'flex-row'">
        <span class="text-[9px] font-mono text-muted tracking-tighter">{{ timeLabel }}</span>
        <div v-if="isOwn" class="flex cursor-pointer hover:opacity-80 transition-opacity" :class="statusClass" @click="isReceiptsModalOpen = true">
          <svg v-if="displayStatus === 'read'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
            <polyline points="22 10 13 19 9 15" />
          </svg>
          <svg v-else-if="displayStatus === 'delivered'" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
            <polyline points="22 10 13 19 9 15" />
          </svg>
          <svg v-else class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
      </div>
    </div>
    
    <ContextMenu
      v-if="showMenu"
      :items="menuItems"
      :emojis="QUICK_REACTIONS"
      :x="menuX"
      :y="menuY"
      @emoji="toggleReaction"
      @close="showMenu = false"
    />

    <ReadReceiptsModal
      v-if="isReceiptsModalOpen"
      :is-open="isReceiptsModalOpen"
      :chat-uuid="message.chat_uuid"
      :message-uuid="message.uuid"
      @close="isReceiptsModalOpen = false"
    />
  </div>
</template>
