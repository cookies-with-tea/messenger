<script setup lang="ts">
import { ref, watch } from 'vue'
import { messageApi } from '@/api'
import type { MessageResponseDTO } from '@/types'
import { useMessengerStore } from '@/stores/messengerStore'

const props = defineProps<{
  chatUuid: string
  isOpen: boolean
}>()

const emit = defineEmits(['close', 'select'])

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
    const res = await messageApi.search(props.chatUuid, query.value)
    results.value = res.data || []
  } catch (e) {
    console.error('[Search] Failed:', e)
  } finally {
    loading.value = false
  }
}

let debounceTimer: any = null
watch(query, (newVal) => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    handleSearch()
  }, 300)
})

watch(() => props.isOpen, (isOpen) => {
  if (!isOpen) {
    query.value = ''
    results.value = []
  }
})

const formatTime = (ts: string) => {
  return new Date(ts).toLocaleDateString() + ' ' + new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const handleSelect = (msg: MessageResponseDTO) => {
  // We want to scroll to message in the main chat
  const el = document.getElementById(`msg-${msg.uuid}`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    el.classList.add('animate-highlight')
    setTimeout(() => el.classList.remove('animate-highlight'), 2000)
  }
  emit('select', msg)
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
    <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-start justify-center pt-20 px-4">
      <!-- Backdrop -->
      <div class="fixed inset-0 bg-void/80 backdrop-blur-sm" @click="emit('close')"></div>
      
      <!-- Modal Content -->
      <div class="relative w-full max-w-lg glass-heavy border border-white/10 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[70vh]">
        <div class="p-4 border-b border-white/5 flex items-center gap-3">
          <svg class="w-5 h-5 text-text-dim" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd" />
          </svg>
          <input 
            v-model="query"
            type="text" 
            placeholder="Search messages..." 
            class="flex-1 bg-transparent border-none text-text-bright placeholder:text-text-dim outline-none text-sm"
            autofocus
            @keydown.esc="emit('close')"
          />
          <button @click="emit('close')" class="p-1 text-text-dim hover:text-text-bright">
            <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-2 space-y-1">
          <div v-if="loading" class="p-8 text-center">
            <div class="w-6 h-6 border-2 border-pulse border-t-transparent rounded-full animate-spin mx-auto mb-2"></div>
            <p class="text-xs text-text-dim uppercase tracking-widest font-mono">Searching neural archives...</p>
          </div>
          
          <div v-else-if="results.length > 0" v-for="msg in results" :key="msg.uuid" 
            @click="handleSelect(msg)"
            class="p-3 rounded-xl hover:bg-white/5 cursor-pointer transition-all group border border-transparent hover:border-white/5"
          >
            <div class="flex items-center justify-between mb-1">
              <span class="text-[10px] font-bold text-pulse uppercase tracking-wider">
                {{ msg.sender?.first_name || 'System' }}
              </span>
              <span class="text-[9px] font-mono text-text-dim">{{ formatTime(msg.created_at) }}</span>
            </div>
            <p class="text-xs text-text-dim group-hover:text-text-bright transition-colors line-clamp-2 leading-relaxed">
              {{ msg.body }}
            </p>
          </div>
          
          <div v-else-if="query && !loading" class="p-8 text-center text-text-dim">
            <p class="text-xs uppercase tracking-widest font-mono">No fragments found</p>
          </div>
          
          <div v-else-if="!query" class="p-8 text-center text-text-dim">
             <p class="text-xs uppercase tracking-widest font-mono">Enter keywords to begin search</p>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>
