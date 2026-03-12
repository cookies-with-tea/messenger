import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'

export interface AudioMedia {
  uuid: string
  url: string
  title?: string
}

export const useAudioStore = defineStore('audio', () => {
  const audio = new Audio()
  const currentAudio = ref<AudioMedia | null>(null)
  const isPlaying = ref(false)
  const currentTime = ref(0)
  const duration = ref(0)

  // Sync state with shared media player
  audio.onplay = () => { isPlaying.value = true }
  audio.onpause = () => { isPlaying.value = false }
  audio.onended = () => {
    isPlaying.value = false
    currentAudio.value = null
    currentTime.value = 0
  }
  audio.ontimeupdate = () => {
    currentTime.value = audio.currentTime
  }
  audio.onloadedmetadata = () => {
    duration.value = audio.duration
  }

  function formatTime(s: number) {
    if (isNaN(s)) return '0:00'
    const m = Math.floor(s / 60)
    const ss = Math.floor(s % 60)
    return `${m}:${ss.toString().padStart(2, '0')}`
  }

  const currentFormattedTime = computed(() => formatTime(currentTime.value))
  const totalFormattedTime = computed(() => formatTime(duration.value))

  const progress = computed(() => {
    if (!duration.value) return 0
    return (currentTime.value / duration.value) * 100
  })

  function play(media: AudioMedia) {
    if (currentAudio.value?.uuid === media.uuid) {
      if (!isPlaying.value) {
        audio.play().catch(console.error)
      }
    } else {
      stop()
      currentAudio.value = media
      audio.src = media.url
      audio.play().catch(console.error)
    }
  }

  function pause() {
    audio.pause()
  }

  function resume() {
    if (currentAudio.value) {
      audio.play().catch(console.error)
    }
  }

  function stop() {
    audio.pause()
    audio.currentTime = 0
    currentAudio.value = null
    isPlaying.value = false
  }

  function seek(seconds: number) {
    if (currentAudio.value && duration.value) {
      audio.currentTime = seconds
    }
  }

  function seekPercent(pct: number) {
    if (currentAudio.value && duration.value) {
      audio.currentTime = pct * duration.value
    }
  }

  return {
    currentAudio,
    isPlaying,
    currentTime,
    duration,
    progress,
    currentFormattedTime,
    totalFormattedTime,
    play,
    pause,
    resume,
    stop,
    seek,
    seekPercent
  }
})
