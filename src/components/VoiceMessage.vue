<template>
  <div class="voice-message flex items-center gap-3 py-1 px-0.5 min-w-[200px] sm:min-w-[240px]">
    <!-- Play/Pause Button -->
    <button 
      @click="togglePlay" 
      class="shrink-0 w-9 h-9 sm:w-10 sm:h-10 flex items-center justify-center rounded-xl sm:rounded-full bg-white/10 hover:bg-white/20 text-white transition-all active:scale-95 border border-white/10 backdrop-blur-md"
    >
      <Transition name="scale" mode="out-in">
        <svg v-if="!isCurrentPlaying" class="w-5 h-5 ml-0.5" viewBox="0 0 20 20" fill="currentColor">
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
          :class="isCurrent && i / waveData.length < progress ? (isOwn ? 'bg-white' : 'bg-pulse') : 'bg-white/20'"
          :style="{ height: `${h}%` }"
        ></div>
      </div>
      
      <div class="flex items-center justify-between px-0.5">
        <span class="text-[9px] font-mono text-white/50 tabular-nums">
          {{ isCurrent ? currentFormattedTime : '0:00' }}
        </span>
        <span class="text-[9px] font-mono text-white/50 tabular-nums">
          {{ isCurrent ? totalFormattedTime : '--:--' }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAudioStore } from '@/stores/audioStore'

const props = withDefaults(defineProps<{
  uuid?: string
  src?: string
  title?: string
  isOwn?: boolean
}>(), {
  uuid: '',
  src: '',
  title: 'Voice Message',
  isOwn: false
})

const audioStore = useAudioStore()

const isCurrent = computed(() => audioStore.currentAudio?.uuid === props.uuid)
const isCurrentPlaying = computed(() => isCurrent.value && audioStore.isPlaying)
const progress = computed(() => isCurrent.value ? audioStore.progress / 100 : 0)

const currentFormattedTime = computed(() => formatTime(audioStore.currentTime))
const totalFormattedTime = computed(() => formatTime(audioStore.duration))

// Generate some pseudo-random wave data for the visualization (seeded by uuid if possible)
const waveData = computed(() => {
  const seed = props.uuid.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
  return Array.from({ length: 30 }, (_, i) => {
    const x = Math.sin(seed + i) * 10000
    return 30 + (x - Math.floor(x)) * 70
  })
})

function formatTime(seconds: number) {
  if (isNaN(seconds)) return '0:00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function togglePlay() {
  if (isCurrent.value) {
    if (audioStore.isPlaying) {
      audioStore.pause()
    } else {
      audioStore.resume()
    }
  } else {
    audioStore.play({
      uuid: props.uuid,
      url: props.src,
      title: props.title || 'Voice Message'
    })
  }
}
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
