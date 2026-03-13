<script setup lang="ts">
import { ref, watch } from 'vue'
import { messageApi } from '@/api'
import type { MessageResponseDTO } from '@/types'
import { useMessengerStore } from '@/stores/messengerStore'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close'])

const store = useMessengerStore()
const query = ref('')
const results = ref<MessageResponseDTO[]>([])
const loading = ref(false)

const handleSearch = async () => {
  if (!query.value.trim()) {
    results.value = []
    return
  }
  
  loading.value = true
  try {
    const res = await messageApi.globalSearch(query.value)
    results.value = res.data || []
  } catch (e) {
    console.error('[GlobalSearch] Failed:', e)
  } finally {
    loading.value = false
  }
}

let debounceTimer: any = null
watch(query, () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    handleSearch()
  }, 300)
})

watch(() => props.isOpen, (val) => {
  if (!val) {
    query.value = ''
    results.value = []
  }
})

const formatTime = (ts: string) => {
  return new Date(ts).toLocaleDateString() + ' ' + new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const handleSelect = async (msg: MessageResponseDTO) => {
  // 1. Select the chat
  await store.selectChat(msg.chat_uuid)
  
  // 2. We want to scroll to message in the main chat
  // Need to wait for messages to load if they aren't already
  setTimeout(() => {
    const el = document.getElementById(`msg-${msg.uuid}`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' })
      el.classList.add('animate-highlight')
      setTimeout(() => el.classList.remove('animate-highlight'), 2000)
    }
  }, 500) // Give it some time to load/render

  emit('close')
}
</script>

<template>
  <Transition
    enter-active-class="transition duration-200 ease-out"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-150 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div v-if="props.isOpen" class="fixed inset-0 z-100 flex items-start justify-center pt-20 px-4">
      <!-- Backdrop -->
      <div class="fixed inset-0 bg-void/80 backdrop-blur-sm" @click="emit('close')"></div>
      
      <!-- Modal Content -->
      <div class="relative w-full max-w-lg glass-heavy border border-white/10 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[70vh]">
        <div class="p-4 border-b border-white/5 flex items-center gap-3">
          <svg class="w-5 h-5 text-pulse" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11zM2 9a7 7 0 1 1 12.452 4.391l3.328 3.329a.75.75 0 1 1-1.06 1.06l-3.329-3.328A7 7 0 0 1 2 9z" clip-rule="evenodd" />
          </svg>
          <input 
            v-model="query"
            type="text" 
            placeholder="Global Archive Search..." 
            class="flex-1 bg-transparent border-none text-text-bright placeholder:text-text-dim outline-none text-sm"
            autofocus
            @keydown.esc="emit('close')"
          />
          <button @click="emit('close')" class="p-1 text-text-dim hover:text-text-bright">
            <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 0 1 1.414 0L10 8.586l4.293-4.293a1 1 0 1 1 1.414 1.414L11.414 10l4.293 4.293a1 1 0 0 1-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 0 1-1.414-1.414L8.586 10 4.293 5.707a1 1 0 0 1 0-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-2 space-y-1">
          <div v-if="loading" class="p-8 text-center">
            <div class="w-6 h-6 border-2 border-pulse border-t-transparent rounded-full animate-spin mx-auto mb-2"></div>
            <p class="text-xs text-text-dim uppercase tracking-widest font-mono">Deep scanning neural network...</p>
          </div>
          
          <div v-else-if="results.length > 0" v-for="msg in results" :key="msg.uuid" 
            @click="handleSelect(msg)"
            class="p-3 rounded-xl hover:bg-white/5 cursor-pointer transition-all group border border-transparent hover:border-white/5"
          >
            <div class="flex items-center justify-between mb-1">
              <div class="flex flex-col">
                <span class="text-[10px] font-bold text-pulse uppercase tracking-wider">
                  {{ msg.sender?.first_name || 'System' }}
                </span>
                <span class="text-[8px] text-text-dim uppercase tracking-tighter">
                   In Archive Flow
                </span>
              </div>
              <span class="text-[9px] font-mono text-text-dim">{{ formatTime(msg.created_at) }}</span>
            </div>
            <p class="text-xs text-text-dim group-hover:text-text-bright transition-colors line-clamp-2 leading-relaxed">
              {{ msg.body }}
            </p>
          </div>
          
          <div v-else-if="query && !loading" class="p-8 text-center text-text-dim">
            <p class="text-xs uppercase tracking-widest font-mono">No matching signals found</p>
          </div>
          
          <div v-else-if="!query" class="p-8 text-center text-text-dim">
             <p class="text-xs uppercase tracking-widest font-mono">Initiate global query...</p>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>
