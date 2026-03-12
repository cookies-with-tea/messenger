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
          class="w-full max-w-sm glass rounded-3xl overflow-hidden border border-white/10 shadow-2xl relative"
        >
          <!-- Hero Section -->
          <div class="h-32 bg-linear-to-br from-ember/30 to-abyss relative">
             <button 
                @click="store.closeProfile"
                class="absolute top-4 right-4 p-2 rounded-full bg-white/5 text-text-dim hover:text-text-bright hover:bg-white/10 transition-all z-10"
              >
                <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
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
              </template>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import { useCallStore } from '@/stores/callStore'
import ChatAvatar from './ChatAvatar.vue'

const store = useMessengerStore()
const callStore = useCallStore()

const displayName = computed(() => {
  const p = store.selectedProfile
  if (!p) return '...'
  return `${p.first_name || ''} ${p.second_name || ''} ${p.last_name || ''}`.replace(/\s+/g, ' ').trim() || 'User'
})

const statusText = computed(() => {
  const p = store.selectedProfile
  if (!p) return ''
  if (p.is_online) return 'Online'
  return formatLastSeen(p.last_seen_at)
})

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
        // Find existing direct chat with this user
        const existing = store.chats.find(c => 
            c.chat_type === 'direct' && 
            c.sender?.uuid === store.selectedProfile?.uuid
        )
        if (existing) {
            store.selectChat(existing.uuid)
            store.closeProfile()
        } else {
            // Future: create new direct chat
            console.log('Should create new direct chat')
            store.closeProfile()
        }
    }
}

function callUser() {
    if (store.selectedProfile) {
        // Similar to startDirectChat, we need a chat ID to start a call
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
