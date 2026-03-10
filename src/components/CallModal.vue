<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
    <div class="bg-surface border border-border rounded-2xl w-full max-w-sm p-6 shadow-glow overflow-hidden relative">
      
      <!-- Background pulse for ringing -->
      <div v-if="callState === 'ringing'" class="absolute inset-0 flex items-center justify-center pointer-events-none opacity-20">
        <div class="w-48 h-48 bg-pulse rounded-full animate-ping"></div>
      </div>

      <div class="relative z-10 flex flex-col items-center text-center">
        
        <!-- Avatar Area -->
        <div class="relative mb-6">
          <div class="w-24 h-24 rounded-full bg-elevated border border-border flex items-center justify-center overflow-hidden">
            <span class="text-3xl font-mono text-pulse uppercase">{{ initials }}</span>
          </div>
          <div v-if="callState === 'connected'" class="absolute bottom-0 right-0 w-6 h-6 bg-sage rounded-full border-4 border-surface"></div>
          <div v-else-if="callState !== 'idle'" class="absolute bottom-0 right-0 w-6 h-6 bg-pulse rounded-full border-4 border-surface animate-bounce"></div>
        </div>

        <!-- Call Info -->
        <h2 class="text-xl font-bold text-text-base mb-1">{{ displayName }}</h2>
        <p class="text-sm font-mono text-muted mb-8">
          <span v-if="callState === 'calling'">Calling...</span>
          <span v-else-if="callState === 'ringing'">Incoming audio call...</span>
          <span v-else-if="callState === 'connected'">00:00</span>
          <span v-else>Connecting...</span>
        </p>

        <!-- Call Controls -->
        <div class="flex items-center gap-6 justify-center w-full">
          
          <!-- Incoming Call Controls -->
          <template v-if="callState === 'ringing' && !isCaller">
            <button @click="rejectCall" class="w-14 h-14 rounded-full bg-error hover:bg-error/90 text-white flex items-center justify-center transition-transform hover:scale-105">
              <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-3.33-2.67m-2.67-3.34a19.79 19.79 0 0 1-3.07-8.63A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91" />
                <line x1="23" x2="1" y1="1" y2="23" />
              </svg>
            </button>
            <button @click="answerCall" class="w-14 h-14 rounded-full bg-sage hover:bg-sage/90 text-white flex items-center justify-center shadow-lg shadow-sage/30 transition-transform hover:scale-110 animate-pulse">
              <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" />
              </svg>
            </button>
          </template>

          <!-- Ongoing/Outgoing Call Controls -->
          <template v-else>
            <button v-if="callState === 'connected'" @click="toggleMute" :class="isMuted ? 'bg-surface border-border text-muted' : 'bg-elevated border-transparent text-text-base'" class="w-12 h-12 rounded-full border flex items-center justify-center transition-colors">
              <svg v-if="!isMuted" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <line x1="12" x2="12" y1="19" y2="22" />
              </svg>
              <svg v-else class="w-5 h-5 text-error" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="1" x2="23" y1="1" y2="23" />
                <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6" />
                <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23" />
                <line x1="12" x2="12" y1="19" y2="22" />
              </svg>
            </button>
            <button @click="endCall" class="w-14 h-14 rounded-full bg-error hover:bg-error/90 text-white flex items-center justify-center transition-transform hover:scale-105">
              <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-3.33-2.67m-2.67-3.34a19.79 19.79 0 0 1-3.07-8.63A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91" />
                <line x1="23" x2="1" y1="1" y2="23" />
              </svg>
            </button>
          </template>

        </div>
      </div>
    </div>
    
    <!-- Hidden audio element for playing remote stream -->
    <audio ref="remoteAudioElement" autoplay></audio>
  </div>
</template>

<script setup lang="ts">
import { computed, watch, ref } from 'vue'
import { useCallStore } from '@/stores/callStore'
import { useMessengerStore } from '@/stores/messengerStore'

const callStore = useCallStore()
const messengerStore = useMessengerStore()

const remoteAudioElement = ref<HTMLAudioElement | null>(null)

const isOpen = computed(() => callStore.callState !== 'idle')
const callState = computed(() => callStore.callState)
const isCaller = computed(() => callStore.isCaller)
const isMuted = computed(() => callStore.isMuted)

// We need to fetch the other user's info to display their name
const activeChat = computed(() => {
  return messengerStore.chats.find(c => c.uuid === callStore.activeCallChatId)
})

const displayName = computed(() => {
  if (!activeChat.value) return 'Unknown'
  if (activeChat.value.chat_type === 'group') return activeChat.value.name ?? 'Group'
  
  const sender = activeChat.value.sender
  if (sender) {
    return [sender.first_name, sender.second_name].filter(Boolean).join(' ') || sender.uuid.split('-')[0]
  }
  return 'User'
})

const initials = computed(() => {
  const name = displayName.value || '?'
  return name.slice(0, 2).toUpperCase()
})

const answerCall = () => callStore.answerCall()
const rejectCall = () => callStore.rejectCall()
const endCall = () => callStore.endCall()
const toggleMute = () => callStore.toggleMute()

// Bind remote stream to audio element whenever it updates
watch(() => callStore.remoteStream, (stream) => {
  if (remoteAudioElement.value) {
    if (stream) {
      remoteAudioElement.value.srcObject = stream
    } else {
      remoteAudioElement.value.srcObject = null
    }
  }
}, { immediate: true })

</script>
