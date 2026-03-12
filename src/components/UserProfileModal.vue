<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import { useCallStore } from '@/stores/callStore'
import ChatAvatar from './ChatAvatar.vue'
import SharedMediaView from './SharedMediaView.vue'

const emit = defineEmits(['openEdit'])
const store = useMessengerStore()
const callStore = useCallStore()

const isMe = computed(() => store.selectedProfile?.uuid === store.currentUserId)

const directChat = computed(() => {
  if (!store.selectedProfile) return null
  return store.chats.find(c => 
    c.chat_type === 'direct' && 
    c.sender?.uuid === store.selectedProfile?.uuid
  )
})

const originalName = computed(() => {
  const p = store.selectedProfile
  if (!p) return '...'
  return `${p.first_name || ''} ${p.second_name || ''} ${p.last_name || ''}`.replace(/\s+/g, ' ').trim() || 'User'
})

const displayName = computed(() => {
  if (directChat.value?.alias) return directChat.value.alias
  return originalName.value
})

const hasAlias = computed(() => !!directChat.value?.alias)

const statusText = computed(() => {
  const p = store.selectedProfile
  if (!p) return ''
  if (p.is_online) return 'Online'
  return formatLastSeen(p.last_seen_at)
})

const newAlias = ref('')
const savingAlias = ref(false)

watch(() => directChat.value, (chat) => {
  newAlias.value = chat?.alias || ''
  if (chat && store.isProfileModalOpen) {
    store.fetchMediaCounts(chat.uuid)
  }
}, { immediate: true })

watch(() => store.isProfileModalOpen, (isOpen) => {
  if (isOpen && directChat.value) {
    store.fetchMediaCounts(directChat.value.uuid)
  }
})

const activeMediaType = ref<'image' | 'video' | 'audio' | null>(null)
const counts = computed(() => {
  if (!directChat.value) return null
  return store.mediaCounts.get(directChat.value.uuid)
})

function showMedia(type: 'image' | 'video' | 'audio') {
  if (!directChat.value) return
  activeMediaType.value = type
  store.fetchSharedMedia(directChat.value.uuid, type)
}

async function saveAlias() {
  if (!directChat.value) return
  savingAlias.value = true
  try {
    await store.updateContactAlias(directChat.value.uuid, newAlias.value || null)
  } finally {
    savingAlias.value = false
  }
}

function formatLastSeen(ts?: string) {
  if (!ts) return 'Offline'
  const date = new Date(ts)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  if (diff < 60_000) return 'last seen just now'
  if (diff < 3600_000) return `last seen ${Math.floor(diff / 60_000)}m ago`
  if (diff < 86400_000) return `last seen ${Math.floor(diff / 3600_000)}h ago`
  return `last seen ${date.toLocaleDateString()}`
}

function startDirectChat() {
    if (store.selectedProfile) {
        const existing = store.chats.find(c => 
            c.chat_type === 'direct' && 
            c.sender?.uuid === store.selectedProfile?.uuid
        )
        if (existing) {
            store.selectChat(existing.uuid)
            store.closeProfile()
        } else {
            console.log('Should create new direct chat')
            store.closeProfile()
        }
    }
}

function callUser() {
    if (store.selectedProfile) {
        const existing = store.chats.find(c => 
            c.chat_type === 'direct' && 
            c.sender?.uuid === store.selectedProfile?.uuid
        )
        if (existing) {
            callStore.startCall(existing.uuid)
            store.closeProfile()
        }
    }
}
</script>

<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="opacity-0"
    enter-to-class="opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="opacity-100"
    leave-to-class="opacity-0"
  >
    <div
      v-if="store.isProfileModalOpen"
      class="fixed inset-0 z-100 flex items-center justify-center p-4 bg-void/60 backdrop-blur-md"
      @click.self="store.closeProfile"
    >
      <Transition
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="opacity-0 scale-95 translate-y-4"
        enter-to-class="opacity-100 scale-100 translate-y-0"
        leave-active-class="transition duration-200 ease-in"
        leave-from-class="opacity-100 scale-100 translate-y-0"
        leave-to-class="opacity-0 scale-95 translate-y-4"
      >
        <div
          v-if="store.isProfileModalOpen"
          class="w-full max-w-sm glass rounded-3xl overflow-y-auto border border-white/10 shadow-2xl relative max-h-[85vh] scrollbar-none"
        >
          <!-- Sticky Header for Close Button -->
          <div class="sticky top-0 z-50 p-4 flex justify-end pointer-events-none">
             <button 
                @click="store.closeProfile"
                class="p-2 rounded-full bg-void/40 backdrop-blur-md text-text-dim hover:text-text-bright hover:bg-void/60 transition-all border border-white/5 pointer-events-auto"
              >
                <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
          </div>

          <!-- Hero Section -->
          <div class="h-32 bg-linear-to-br from-ember/30 to-abyss relative -mt-16">
          </div>

          <!-- Profile Info -->
          <div class="px-6 pb-8 -mt-16 relative">
            <div class="flex flex-col items-center text-center">
              <div class="relative">
                <ChatAvatar 
                  :chat="store.selectedProfile" 
                  :size="110" 
                  class="border-4 border-abyss shadow-2xl" 
                />
                <div 
                  v-if="store.selectedProfile?.is_online"
                  class="absolute bottom-2 right-2 w-5 h-5 rounded-full bg-sage border-4 border-abyss shadow-[0_0_10px_rgba(var(--color-sage),0.5)]"
                ></div>
              </div>

              <template v-if="store.profileLoading">
                <div class="mt-4 h-6 w-32 bg-white/5 animate-pulse rounded-full"></div>
                <div class="mt-2 h-4 w-48 bg-white/5 animate-pulse rounded-full"></div>
              </template>
              
              <template v-else-if="store.selectedProfile">
                <h2 class="mt-4 text-2xl font-black font-syne text-text-bright tracking-tight uppercase">
                  {{ displayName }}
                </h2>
                <div v-if="hasAlias" class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest -mt-1">
                  Original: {{ originalName }}
                </div>
                <p class="text-xs font-mono text-text-dim uppercase tracking-widest mt-1">
                   {{ statusText }}
                </p>

                <!-- Details List -->
                <div class="w-full mt-8 space-y-4">
                  <div class="flex flex-col items-start gap-1">
                    <span class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest">Email Address</span>
                    <span class="text-sm text-text-bright font-medium">{{ store.selectedProfile.email || '—' }}</span>
                  </div>
                  
                  <div v-if="store.selectedProfile.phone" class="flex flex-col items-start gap-1 border-t border-white/5 pt-4">
                    <span class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest">Phone Number</span>
                    <span class="text-sm text-text-bright font-medium">{{ store.selectedProfile.phone }}</span>
                  </div>

                   <div v-if="store.selectedProfile.city || store.selectedProfile.street" class="flex flex-col items-start gap-1 border-t border-white/5 pt-4">
                    <span class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest">Location</span>
                    <span class="text-sm text-text-bright font-medium">
                        {{ [store.selectedProfile.city, store.selectedProfile.street].filter(Boolean).join(', ') }}
                    </span>
                  </div>
                </div>

                <!-- Action Buttons -->
                <div class="w-full mt-8 grid grid-cols-2 gap-3">
                  <button 
                    @click="startDirectChat"
                    class="flex items-center justify-center gap-2 py-3 rounded-2xl bg-white/5 hover:bg-white/10 text-text-bright font-bold text-xs uppercase transition-all"
                  >
                    <span>Message</span>
                  </button>
                  <button 
                    @click="callUser"
                    class="flex items-center justify-center gap-2 py-3 rounded-2xl bg-ember/20 hover:bg-ember/30 text-ember font-bold text-xs uppercase transition-all"
                  >
                    <span>Call</span>
                  </button>
                </div>

                <!-- Edit/Rename Section -->
                <div class="w-full mt-4 border-t border-white/5 pt-6">
                  <template v-if="isMe">
                    <button 
                      @click="$emit('openEdit')"
                      class="w-full flex items-center justify-center gap-2 py-4 rounded-2xl bg-white/10 hover:bg-white/15 text-text-bright font-black text-xs uppercase transition-all tracking-widest border border-white/5 shadow-xl"
                    >
                      <i class="fas fa-edit mr-2"></i>
                      <span>Edit My Profile</span>
                    </button>
                  </template>
                  <template v-else-if="directChat">
                    <div class="flex flex-col gap-2">
                       <span class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest text-left">Contact Alias</span>
                       <div class="flex gap-2">
                          <input 
                            v-model="newAlias" 
                            placeholder="Set nickname..." 
                            class="flex-1 bg-white/5 border border-white/10 rounded-xl px-4 py-2 text-sm text-text-bright focus:outline-none focus:border-ember transition-all"
                          />
                          <button 
                            @click="saveAlias"
                            :disabled="savingAlias"
                            class="px-4 py-2 rounded-xl bg-sage/20 text-sage hover:bg-sage/30 text-xs font-bold uppercase transition-all disabled:opacity-50"
                          >
                            {{ savingAlias ? '...' : 'Save' }}
                          </button>
                       </div>
                    </div>
                  </template>
                </div>

                <!-- Shared Media Section -->
                <div v-if="directChat" class="w-full mt-8 border-t border-white/5 pt-6">
                   <div class="flex items-center justify-between mb-4">
                      <span class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest">Shared Media</span>
                   </div>
                   <div class="grid grid-cols-3 gap-3">
                      <button 
                        @click="showMedia('image')"
                        class="flex flex-col items-center gap-2 p-3 rounded-2xl bg-white/5 hover:bg-white/10 transition-all border border-white/5"
                      >
                         <span class="text-xl">🖼️</span>
                         <div class="flex flex-col items-center">
                            <span class="text-sm font-bold text-text-bright">{{ counts?.images ?? 0 }}</span>
                            <span class="text-[8px] font-mono text-text-dim uppercase">Photos</span>
                         </div>
                      </button>
                      <button 
                        @click="showMedia('video')"
                        class="flex flex-col items-center gap-2 p-3 rounded-2xl bg-white/5 hover:bg-white/10 transition-all border border-white/5"
                      >
                         <span class="text-xl">🎥</span>
                         <div class="flex flex-col items-center">
                            <span class="text-sm font-bold text-text-bright">{{ counts?.videos ?? 0 }}</span>
                            <span class="text-[8px] font-mono text-text-dim uppercase">Videos</span>
                         </div>
                      </button>
                      <button 
                        @click="showMedia('audio')"
                        class="flex flex-col items-center gap-2 p-3 rounded-2xl bg-white/5 hover:bg-white/10 transition-all border border-white/5"
                      >
                         <span class="text-xl">🎵</span>
                         <div class="flex flex-col items-center">
                            <span class="text-sm font-bold text-text-bright">{{ counts?.audio ?? 0 }}</span>
                            <span class="text-[8px] font-mono text-text-dim uppercase">Audio</span>
                         </div>
                      </button>
                   </div>
                </div>
              </template>
            </div>
          </div>

          <!-- Shared Media View Overlay -->
          <Transition
            enter-active-class="transition duration-300 ease-out"
            enter-from-class="translate-x-full"
            enter-to-class="translate-x-0"
            leave-active-class="transition duration-200 ease-in"
            leave-from-class="translate-x-0"
            leave-to-class="translate-x-full"
          >
            <div v-if="activeMediaType && directChat" class="absolute inset-0 z-100">
               <SharedMediaView 
                  :chat-uuid="directChat.uuid" 
                  :active-type="activeMediaType" 
                  @back="activeMediaType = null"
               />
            </div>
          </Transition>
        </div>
      </Transition>
    </div>
  </Transition>
</template>
