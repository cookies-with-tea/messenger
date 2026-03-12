<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="-translate-y-full opacity-0"
    enter-to-class="translate-y-0 opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="translate-y-0 opacity-100"
    leave-to-class="-translate-y-full opacity-0"
  >
    <div 
      v-if="audioStore.currentAudio"
      class="fixed top-0 left-0 right-0 z-50 w-full"
    >
      <!-- Progress Bar Background (Full Width) -->
      <div 
        class="absolute bottom-0 left-0 right-0 h-1 bg-white/5 cursor-pointer"
        @click="seek"
      >
        <div 
          class="h-full bg-ember shadow-[0_0_10px_rgba(255,107,0,0.5)] transition-all duration-100"
          :style="{ width: `${audioStore.progress}%` }"
        ></div>
      </div>

      <div class="glass border-b border-white/10 shadow-lg px-4 py-2 flex items-center justify-between gap-4">
        <!-- Info & Controls -->
        <div class="flex items-center gap-4 flex-1 min-w-0">
          <button 
            @click="togglePlay"
            class="w-8 h-8 shrink-0 flex items-center justify-center rounded-full bg-ember/20 text-ember hover:bg-ember/30 transition-all border border-ember/20"
          >
            <svg v-if="audioStore.isPlaying" class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
            </svg>
            <svg v-else class="w-4 h-4 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z" />
            </svg>
          </button>

          <div class="flex flex-col min-w-0">
            <span class="text-[10px] font-mono text-ember font-bold uppercase tracking-widest truncate">
              {{ audioStore.currentAudio.title || 'Voice Message' }}
            </span>
            <span class="text-[9px] font-mono text-text-dim tabular-nums">
              {{ audioStore.currentFormattedTime }} / {{ audioStore.totalFormattedTime }}
            </span>
          </div>
        </div>

        <!-- Right Controls -->
        <div class="flex items-center gap-2">
          <button 
            @click="audioStore.stop"
            class="p-2 text-text-dim hover:text-ember transition-colors"
            title="Close"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useAudioStore } from '@/stores/audioStore'

const audioStore = useAudioStore()

function togglePlay() {
  if (audioStore.isPlaying) {
    audioStore.pause()
  } else {
    audioStore.resume()
  }
}

function seek(e: MouseEvent) {
  const el = e.currentTarget as HTMLElement
  const rect = el.getBoundingClientRect()
  const x = e.clientX - rect.left
  const pct = x / rect.width
  audioStore.seekPercent(pct)
}
</script>
