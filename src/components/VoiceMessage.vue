<template>
  <div class="voice-message flex items-center gap-3 py-1 px-0.5 min-w-[200px] sm:min-w-[240px]">
    <!-- Play/Pause Button -->
    <button 
      @click="togglePlay" 
      class="shrink-0 w-9 h-9 sm:w-10 sm:h-10 flex items-center justify-center rounded-xl sm:rounded-full bg-white/10 hover:bg-white/20 text-white transition-all active:scale-95 border border-white/10 backdrop-blur-md"
    >
      <Transition name="scale" mode="out-in">
        <svg v-if="!isPlaying" class="w-5 h-5 ml-0.5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
        </svg>
        <svg v-else class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
        </svg>
      </Transition>
    </button>

    <!-- Waveform & Time -->
    <div class="flex-1 flex flex-col gap-1 overflow-hidden">
      <!-- Custom Waveform (Static or Animated) -->
      <div class="waveform flex items-center gap-0.5 h-6">
        <div 
          v-for="(h, i) in waveData" 
          :key="i"
          class="w-0.5 rounded-full transition-all duration-300"
          :class="i / waveData.length < progress ? (isOwn ? 'bg-white' : 'bg-pulse') : 'bg-white/20'"
          :style="{ height: `${h}%` }"
        ></div>
      </div>
      
      <div class="flex items-center justify-between px-0.5">
        <span class="text-[9px] font-mono text-white/50 tabular-nums">{{ currentFormattedTime }}</span>
        <span class="text-[9px] font-mono text-white/50 tabular-nums">{{ totalFormattedTime }}</span>
      </div>
    </div>

    <!-- Hidden Audio Element -->
    <audio 
      ref="audioRef" 
      :src="src" 
      @timeupdate="onTimeUpdate" 
      @ended="onEnded"
      @loadedmetadata="onLoadedMetadata"
      preload="metadata"
    ></audio>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'

const props = defineProps<{
  src: string
  isOwn?: boolean
}>()

const audioRef = ref<HTMLAudioElement>()
const isPlaying = ref(false)
const progress = ref(0)
const currentTime = ref(0)
const duration = ref(0)

// Generate some pseudo-random wave data for the visualization
const waveData = Array.from({ length: 30 }, () => 30 + Math.random() * 70)

const currentFormattedTime = computed(() => formatTime(currentTime.value))
const totalFormattedTime = computed(() => formatTime(duration.value))

function formatTime(seconds: number) {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function togglePlay() {
  if (!audioRef.value) return
  
  if (isPlaying.value) {
    audioRef.value.pause()
  } else {
    audioRef.value.play().catch(() => {
      isPlaying.value = false
    })
  }
  isPlaying.value = !isPlaying.value
}

function onTimeUpdate() {
  if (!audioRef.value || !audioRef.value.duration) return
  currentTime.value = audioRef.value.currentTime
  progress.value = audioRef.value.currentTime / audioRef.value.duration
}

function onEnded() {
  isPlaying.value = false
  progress.value = 0
  currentTime.value = 0
}

function onLoadedMetadata() {
  if (!audioRef.value) return
  duration.value = audioRef.value.duration
}

onUnmounted(() => {
  if (audioRef.value) {
    audioRef.value.pause()
  }
})
</script>

<style scoped>
.scale-enter-active, .scale-leave-active {
  transition: all 0.2s ease;
}
.scale-enter-from, .scale-leave-to {
  opacity: 0;
  transform: scale(0.5);
}

.waveform div {
  min-height: 2px;
}
</style>
