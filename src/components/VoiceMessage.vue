<template>
  <div class="voice-message flex items-center gap-3 p-2 rounded-2xl bg-surface/50 border border-border/50 max-w-[280px]">
    <!-- Play/Pause Button -->
    <button 
      @click="togglePlay" 
      class="shrink-0 w-10 h-10 flex items-center justify-center rounded-full bg-pulse text-white hover:shadow-glow transition-all active:scale-95"
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
          :class="i / waveData.length < progress ? 'bg-pulse' : 'bg-text-dim/30'"
          :style="{ height: `${h}%` }"
        ></div>
      </div>
      
      <div class="flex items-center justify-between">
        <span class="text-[10px] font-mono text-text-dim tabular-nums">{{ currentFormattedTime }}</span>
        <span class="text-[10px] font-mono text-text-dim tabular-nums">{{ totalFormattedTime }}</span>
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
    audioRef.value.play()
  }
  isPlaying.value = !isPlaying.value
}

function onTimeUpdate() {
  if (!audioRef.value) return
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

.shadow-glow {
  box-shadow: 0 0 15px rgba(var(--color-pulse-rgb), 0.4);
}

.waveform div {
  min-height: 2px;
}
</style>
