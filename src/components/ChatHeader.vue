<template>
  <div class="flex items-center gap-3 px-4 sm:px-6 py-3 sm:py-4 border-b border-white/5 glass backdrop-blur-md relative z-20">
    <!-- Back Button for Mobile -->
    <button
      @click="goBack"
      class="md:hidden p-1.5 sm:p-2 -ml-1 sm:-ml-2 rounded-lg text-text-dim hover:text-text-bright hover:bg-white/5 transition-all"
    >
      <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
      </svg>
    </button>

    <ChatAvatar :chat="chatSender" :size="32" class="sm:w-[38px] sm:h-[38px]" :show-status="isDirect" />

    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <h2 class="text-xs sm:text-sm font-syne font-black text-text-bright truncate leading-tight tracking-tight uppercase">
          {{ displayName }}
        </h2>
        <div v-if="props.chat.sender?.is_online" class="w-1.5 h-1.5 rounded-full bg-sage shadow-[0_0_8px_rgba(var(--color-sage),0.8)] animate-pulse"></div>
      </div>
      <p class="text-[9px] sm:text-[10px] font-mono truncate uppercase tracking-widest mt-0.5" :class="statusColorClass">
        {{ statusText }}
      </p>
    </div>

    <div class="flex items-center gap-1 sm:gap-2">
      <button
        v-for="(action, i) in actions"
        :key="i"
        @click="handleAction(action.action)"
        class="p-2 sm:p-2.5 rounded-xl text-text-dim hover:text-text-bright hover:bg-white/5 transition-all relative group"
        :title="action.label"
      >
        <component :is="action.icon" class="w-3.5 h-3.5 sm:w-4 sm:h-4 group-hover:scale-110 transition-transform" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h } from 'vue'
import { useRouter } from 'vue-router'
import { useMessengerStore } from '@/stores/messengerStore'
import { useCallStore } from '@/stores/callStore'
import type { ChatResponseDTO } from '@/types'
import ChatAvatar from './ChatAvatar.vue'

const props = defineProps<{ chat: ChatResponseDTO }>()
const router = useRouter()
const store = useMessengerStore()

function goBack() {
  store.activeChatId = null
  router.push('/')
}

const isDirect = computed(() => props.chat.chat_type === 'direct')
const chatSender = computed(() => props.chat.sender)

const displayName = computed(() => {
  const sender = props.chat.sender
  return sender ? `${sender.first_name || 'Chat'} ${sender.second_name || ''}`.trim() : (props.chat.name || 'Chat')
})

const statusText = computed(() => {
  const typing = store.typingUsersFor(props.chat.uuid)
  if (typing.length > 0) {
    if (props.chat.chat_type === 'direct') return 'Typing...'
    // For groups, we might want names, but we only have UUIDs in typingMap 
    // unless we look them up in members. For now, simple count or "Someone is typing"
    return typing.length === 1 ? 'Someone is typing...' : `${typing.length} people are typing...`
  }

  if (props.chat.chat_type === 'group') {
    return `${props.chat.member_count ?? '?'} members`
  }
  const sender = props.chat.sender
  if (!sender) return ''
  return sender.is_online ? 'Online' : formatLastSeen(sender.last_seen_at)
})

const statusColorClass = computed(() => {
  const typing = store.typingUsersFor(props.chat.uuid)
  if (typing.length > 0) return 'text-sage font-bold italic'
  return props.chat.sender?.is_online ? 'text-sage' : 'text-text-dim'
})

const PhoneIcon  = defineComponent({ render: () => h('svg', { viewBox: '0 0 20 20', fill: 'currentColor' }, [h('path', { 'fill-rule': 'evenodd', d: 'M2 3.5A1.5 1.5 0 013.5 2h1.148a1.5 1.5 0 011.465 1.175l.716 3.223a1.5 1.5 0 01-1.052 1.767l-.933.267c-.41.117-.643.555-.48.95a11.542 11.542 0 006.254 6.254c.395.163.833-.07.95-.48l.267-.933a1.5 1.5 0 011.767-1.052l3.223.716A1.5 1.5 0 0118 15.352V16.5a1.5 1.5 0 01-1.5 1.5H15c-1.149 0-2.263-.15-3.326-.43A13.022 13.022 0 012.43 8.326 13.019 13.019 0 012 5V3.5z', 'clip-rule': 'evenodd' })]) })
const SearchIcon = defineComponent({ render: () => h('svg', { viewBox: '0 0 20 20', fill: 'currentColor' }, [h('path', { 'fill-rule': 'evenodd', d: 'M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z', 'clip-rule': 'evenodd' })]) })

const callStore = useCallStore()

const actions = [
  { label: 'Voice call',       icon: PhoneIcon,  action: 'call' },
  { label: 'Search messages',  icon: SearchIcon, action: 'search' },
]

function handleAction(actionType: string) {
  if (actionType === 'call') {
    callStore.startCall(props.chat.uuid)
  }
}

function formatLastSeen(ts?: string) {
  if (!ts) return ''
  const date = new Date(ts)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  if (diff < 60_000) return 'just now'
  if (diff < 3600_000) return `${Math.floor(diff / 60_000)}m ago`
  if (diff < 86400_000) return `${Math.floor(diff / 3600_000)}h ago`
  return `${date.toLocaleDateString()}`
}
</script>
