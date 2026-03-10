<template>
  <div class="voice-recorder flex items-center gap-3">
    <!-- Recording State UI -->
    <div v-if="isRecording" class="flex items-center gap-3 px-3 py-2 bg-pulse/10 rounded-xl border border-pulse/30">
      <div class="w-2.5 h-2.5 bg-pulse rounded-full animate-ping shrink-0"></div>
      <span class="text-xs font-mono text-pulse tabular-nums">{{ formattedTime }}</span>

      <!-- Live waveform from mic -->
      <div class="voice-wave flex items-end gap-0.5 h-5">
        <div
          v-for="(h, i) in waveBars"
          :key="i"
          class="w-0.5 bg-pulse rounded-full transition-none"
          :style="{ height: `${h}%`, minHeight: '10%' }"
        ></div>
      </div>

      <button
        @click="cancelRecording"
        class="ml-2 p-1.5 text-text-dim hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all"
        title="Cancel"
      >
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>

      <button
        @click="stopRecording"
        class="p-1.5 bg-pulse text-white rounded-lg hover:shadow-glow transition-all"
        title="Send"
      >
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
        </svg>
      </button>
    </div>

    <!-- Initial State Button -->
    <button
      v-else
      @click="startRecording"
      class="shrink-0 p-2.5 rounded-xl text-text-dim hover:text-pulse hover:bg-elevated transition-all"
      title="Record voice message"
    >
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted, computed } from 'vue'

const BAR_COUNT = 20

const props = defineProps<{
  disabled?: boolean
}>()

const emit = defineEmits<{
  send: [blob: Blob]
}>()

const isRecording = ref(false)
const recordingTime = ref(0)
const waveBars = ref<number[]>(Array(BAR_COUNT).fill(10))

let mediaRecorder: MediaRecorder | null = null
let audioChunks: Blob[] = []
let timerInterval: ReturnType<typeof setInterval> | null = null
let shouldSend = false

// Web Audio API
let audioCtx: AudioContext | null = null
let analyser: AnalyserNode | null = null
let animFrameId: number | null = null

const formattedTime = computed(() => {
  const mins = Math.floor(recordingTime.value / 60)
  const secs = recordingTime.value % 60
  return `${mins}:${secs.toString().padStart(2, '0')}`
})

function startAnalyser(stream: MediaStream) {
  audioCtx = new AudioContext()
  analyser = audioCtx.createAnalyser()
  analyser.fftSize = 64
  const source = audioCtx.createMediaStreamSource(stream)
  source.connect(analyser)

  const dataArray = new Uint8Array(analyser.frequencyBinCount)

  function tick() {
    if (!analyser) return
    analyser.getByteFrequencyData(dataArray)

    // Map frequency bins → bar heights (0–100%)
    const step = Math.floor(dataArray.length / BAR_COUNT)
    waveBars.value = Array.from({ length: BAR_COUNT }, (_, i) => {
      const val = dataArray[i * step] ?? 0
      return Math.max(10, Math.round((val / 255) * 100))
    })

    animFrameId = requestAnimationFrame(tick)
  }
  animFrameId = requestAnimationFrame(tick)
}

function stopAnalyser() {
  if (animFrameId !== null) {
    cancelAnimationFrame(animFrameId)
    animFrameId = null
  }
  if (audioCtx) {
    audioCtx.close()
    audioCtx = null
  }
  analyser = null
  waveBars.value = Array(BAR_COUNT).fill(10)
}

async function startRecording() {
  if (props.disabled) return

  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
    mediaRecorder = new MediaRecorder(stream)
    audioChunks = []

    mediaRecorder.ondataavailable = (event) => {
      audioChunks.push(event.data)
    }

    mediaRecorder.onstop = () => {
      if (shouldSend && audioChunks.length > 0) {
        const audioBlob = new Blob(audioChunks, { type: 'audio/webm' })
        emit('send', audioBlob)
      }
      stream.getTracks().forEach(track => track.stop())
    }

    mediaRecorder.start()
    shouldSend = false
    isRecording.value = true
    recordingTime.value = 0
    timerInterval = setInterval(() => {
      recordingTime.value++
    }, 1000)

    startAnalyser(stream)
  } catch (err) {
    console.error('Error accessing microphone:', err)
    alert('Could not access microphone. Please check permissions.')
  }
}

function stopRecording() {
  if (mediaRecorder && isRecording.value) {
    shouldSend = true
    mediaRecorder.stop()
    cleanup()
  }
}

function cancelRecording() {
  if (mediaRecorder && isRecording.value) {
    shouldSend = false
    mediaRecorder.stop()
    cleanup()
  }
}

function cleanup() {
  isRecording.value = false
  stopAnalyser()
  if (timerInterval) {
    clearInterval(timerInterval)
    timerInterval = null
  }
}

onUnmounted(() => {
  if (mediaRecorder && isRecording.value) {
    mediaRecorder.stop()
  }
  cleanup()
})
</script>

<style scoped>
.shadow-glow {
  box-shadow: 0 0 15px rgba(var(--color-pulse-rgb), 0.4);
}
</style>
