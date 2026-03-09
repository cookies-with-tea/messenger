<template>
  <div class="flex items-center gap-3 px-5 py-3.5 border-b border-border bg-abyss/80 backdrop-blur-sm">
    <ChatAvatar :chat="chat" :size="38" />

    <div class="flex-1 min-w-0">
      <h2 class="text-sm font-syne font-bold text-text-bright truncate">
        {{ chat.name ?? 'Chat' }}
      </h2>
      <p class="text-xs font-mono text-text-dim truncate">
        {{ chat.chat_type === 'group'
            ? `${chat.member_count ?? '?'} members`
            : '' }}
      </p>
    </div>

    <div class="flex items-center gap-1">
      <button
        v-for="(action, i) in actions"
        :key="i"
        class="p-2 rounded-lg text-text-dim hover:text-text-bright hover:bg-elevated transition-all"
        :title="action.label"
      >
        <component :is="action.icon" class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineComponent, h } from 'vue'
import type { ChatResponseDTO } from '@/types'
import ChatAvatar from './ChatAvatar.vue'

defineProps<{ chat: ChatResponseDTO }>()

const PhoneIcon  = defineComponent({ render: () => h('svg', { viewBox: '0 0 20 20', fill: 'currentColor' }, [h('path', { 'fill-rule': 'evenodd', d: 'M2 3.5A1.5 1.5 0 013.5 2h1.148a1.5 1.5 0 011.465 1.175l.716 3.223a1.5 1.5 0 01-1.052 1.767l-.933.267c-.41.117-.643.555-.48.95a11.542 11.542 0 006.254 6.254c.395.163.833-.07.95-.48l.267-.933a1.5 1.5 0 011.767-1.052l3.223.716A1.5 1.5 0 0118 15.352V16.5a1.5 1.5 0 01-1.5 1.5H15c-1.149 0-2.263-.15-3.326-.43A13.022 13.022 0 012.43 8.326 13.019 13.019 0 012 5V3.5z', 'clip-rule': 'evenodd' })]) })
const SearchIcon = defineComponent({ render: () => h('svg', { viewBox: '0 0 20 20', fill: 'currentColor' }, [h('path', { 'fill-rule': 'evenodd', d: 'M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z', 'clip-rule': 'evenodd' })]) })

const actions = [
  { label: 'Voice call',       icon: PhoneIcon },
  { label: 'Search messages',  icon: SearchIcon },
]
</script>
