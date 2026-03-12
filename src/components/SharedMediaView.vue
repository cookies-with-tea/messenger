<script setup lang="ts">
import { computed } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import { useAudioStore } from '@/stores/audioStore'

const props = defineProps<{
  chatUuid: string
  activeType: 'image' | 'video' | 'audio'
}>()

const emit = defineEmits(['back'])

const store = useMessengerStore()
const audioStore = useAudioStore()

const loading = computed(() => store.sharedMediaLoading)
const items = computed(() => store.sharedMedia.get(props.chatUuid + props.activeType) || [])

function toggleAudio(media: any) {
  if (audioStore.currentAudio?.uuid === media.uuid) {
    if (audioStore.isPlaying) {
      audioStore.pause()
    } else {
      audioStore.resume()
    }
  } else {
    audioStore.play({
      uuid: media.uuid,
      url: media.url,
      title: media.title || 'Voice Message'
    })
  }
}

function formatTime(s: number) {
  if (isNaN(s)) return '0:00'
  const m = Math.floor(s / 60)
  const ss = Math.floor(s % 60)
  return `${m}:${ss.toString().padStart(2, '0')}`
}

function openMedia(media: any) {
  if (media?.url) {
    window.open(media.url, '_blank')
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-abyss/95 backdrop-blur-xl">
    <!-- Player Progress Bar (Synced with global store) -->
    <div 
      v-if="audioStore.currentAudio" 
      class="h-1.5 bg-white/5 relative group cursor-pointer overflow-hidden" 
      @click="audioStore.seekPercent($event.clientX / ($event.currentTarget as HTMLElement).getBoundingClientRect().width)"
    >
      <div 
        class="h-full bg-ember shadow-[0_0_15px_rgba(255,107,0,0.6)] transition-all duration-100 ease-out" 
        :style="{ width: `${audioStore.progress}%` }"
      ></div>
    </div>

    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-white/5">
      <div class="flex items-center gap-3">
        <button @click="$emit('back')" class="p-2 hover:bg-white/5 rounded-full text-text-dim transition-all">
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <div class="flex flex-col">
          <span class="text-xs font-black font-syne uppercase tracking-widest text-text-bright">
            Shared {{ activeType }}s
          </span>
          <span v-if="audioStore.currentAudio" class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest animate-pulse">
            {{ formatTime(audioStore.currentTime) }} / {{ formatTime(audioStore.duration) }}
          </span>
        </div>
      </div>

      <!-- Stop Button -->
      <button 
        v-if="audioStore.currentAudio"
        @click="audioStore.stop"
        class="p-2 hover:bg-ember/20 rounded-full text-ember transition-all"
        title="Stop Playback"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="6" width="12" height="12" rx="2" />
        </svg>
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-4 scrollbar-none">
      <div v-if="loading" class="flex items-center justify-center h-full">
        <div class="w-8 h-8 border-4 border-ember border-t-transparent rounded-full animate-spin"></div>
      </div>
      
      <div v-else-if="!items.length" class="flex flex-col items-center justify-center h-full text-text-dim opacity-50">
        <span class="text-4xl mb-4">🏜️</span>
        <p class="text-xs font-mono uppercase tracking-widest">No shared {{ activeType }}s yet</p>
      </div>

      <div v-else class="grid grid-cols-3 gap-2">
        <div 
          v-for="msg in items" 
          :key="msg.uuid"
          class="aspect-square rounded-xl overflow-hidden bg-white/5 border border-white/5 hover:border-ember/50 transition-all cursor-pointer group relative"
          @click="openMedia(msg.media)"
        >
          <template v-if="msg.media?.media_type === 'image'">
            <img :src="msg.media.url" class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110" />
          </template>
          <template v-else-if="msg.media?.media_type === 'video'">
            <div class="w-full h-full flex items-center justify-center bg-void">
              <span class="text-2xl">🎥</span>
              <div class="absolute inset-0 bg-void/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all">
                 <span class="text-white">▶️</span>
              </div>
            </div>
          </template>
          <template v-else-if="msg.media?.media_type === 'audio'">
            <div 
              class="w-full h-full flex flex-col items-center justify-center bg-sage/10 p-2 text-center group/audio relative overflow-hidden"
              @click.stop="toggleAudio(msg.media)"
            >
              <div class="relative z-10 flex flex-col items-center">
                <div class="w-12 h-12 flex items-center justify-center rounded-full bg-sage/20 mb-1 group-hover/audio:bg-sage/30 transition-all">
                  <svg v-if="audioStore.currentAudio?.uuid === msg.media?.uuid && audioStore.isPlaying" class="w-6 h-6 text-sage" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
                  </svg>
                  <svg v-else class="w-6 h-6 text-sage" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M8 5v14l11-7z" />
                  </svg>
                </div>
                <span class="text-[8px] font-mono text-sage truncate w-full px-1">{{ msg.media.title || 'Voice Message' }}</span>
              </div>
              
              <!-- Progress indicator for current playing item -->
              <div 
                v-if="audioStore.currentAudio?.uuid === msg.media?.uuid" 
                class="absolute bottom-0 left-0 h-1 bg-sage/40 transition-all duration-100"
                :style="{ width: `${audioStore.progress}%` }"
              ></div>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
