<template>
  <div class="flex flex-col h-full bg-abyss/95 backdrop-blur-xl">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-white/5">
      <div class="flex items-center gap-3">
        <button @click="$emit('back')" class="p-2 hover:bg-white/5 rounded-full text-text-dim transition-all">
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <span class="text-sm font-black font-syne uppercase tracking-widest text-text-bright">
          Shared {{ activeType }}s
        </span>
      </div>
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
            <div class="w-full h-full flex flex-col items-center justify-center bg-sage/10 p-2 text-center">
              <span class="text-2xl mb-1">🎵</span>
              <span class="text-[8px] font-mono text-sage truncate w-full">{{ msg.media.title || 'Audio' }}</span>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'

const props = defineProps<{
  chatUuid: string
  activeType: 'image' | 'video' | 'audio'
}>()

const emit = defineEmits(['back'])

const store = useMessengerStore()

const loading = computed(() => store.sharedMediaLoading)
const items = computed(() => store.sharedMedia.get(props.chatUuid + props.activeType) || [])

function openMedia(media: any) {
  if (media?.url) {
    window.open(media.url, '_blank')
  }
}
</script>
