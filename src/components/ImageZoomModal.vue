<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  src: string
  isOpen: boolean
}>()

const emit = defineEmits(['close'])

const scale = ref(1)
const translateX = ref(0)
const translateY = ref(0)
const isDragging = ref(false)
const lastMouseX = ref(0)
const lastMouseY = ref(0)

const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  const newScale = Math.max(1, Math.min(scale.value * delta, 5))
  scale.value = newScale
  
  if (scale.value === 1) {
    translateX.value = 0
    translateY.value = 0
  }
}

const startDrag = (e: MouseEvent) => {
  if (scale.value > 1) {
    isDragging.value = true
    lastMouseX.value = e.clientX
    lastMouseY.value = e.clientY
  }
}

const onDrag = (e: MouseEvent) => {
  if (isDragging.value) {
    const dx = e.clientX - lastMouseX.value
    const dy = e.clientY - lastMouseY.value
    translateX.value += dx
    translateY.value += dy
    lastMouseX.value = e.clientX
    lastMouseY.value = e.clientY
  }
}

const stopDrag = () => {
  isDragging.value = false
}

const reset = () => {
  scale.value = 1
  translateX.value = 0
  translateY.value = 0
}

onMounted(() => {
  window.addEventListener('mousemove', onDrag)
  window.addEventListener('mouseup', stopDrag)
})

onUnmounted(() => {
  window.removeEventListener('mousemove', onDrag)
  window.removeEventListener('mouseup', stopDrag)
})
</script>

<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="opacity-0"
    enter-to-class="opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="opacity-100"
    leave-to-class="opacity-0"
  >
    <div v-if="isOpen" class="fixed inset-0 z-[150] flex items-center justify-center p-4 bg-void/95 backdrop-blur-md">
      <!-- Close button on top right -->
      <button 
        @click="emit('close')"
        class="absolute top-6 right-6 p-3 rounded-full bg-white/5 text-text-bright hover:bg-white/10 hover:scale-110 transition-all z-[160]"
      >
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>

      <!-- Zoom Controls overlay -->
      <div class="absolute bottom-10 left-1/2 -translate-x-1/2 flex items-center gap-4 px-6 py-3 rounded-2xl glass border border-white/10 z-[160]">
        <button @click="scale = Math.max(1, scale - 0.5)" class="text-text-dim hover:text-text-bright transition-colors">
           <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M3 10a.75.75 0 01.75-.75h12.5a.75.75 0 010 1.5H3.75A.75.75 0 013 10z" clip-rule="evenodd" /></svg>
        </button>
        <span class="text-xs font-mono text-text-bright min-w-[3rem] text-center">{{ Math.round(scale * 100) }}%</span>
        <button @click="scale = Math.min(5, scale + 0.5)" class="text-text-dim hover:text-text-bright transition-colors">
           <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor"><path d="M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z" /></svg>
        </button>
        <div class="w-px h-4 bg-white/10 mx-2"></div>
        <button @click="reset" class="text-[10px] font-mono uppercase tracking-widest text-text-dim hover:text-pulse transition-colors">Reset</button>
      </div>

      <!-- Image -->
      <div 
        class="relative w-full h-full flex items-center justify-center overflow-hidden cursor-move"
        @mousedown="startDrag"
        @wheel="handleWheel"
      >
        <img 
          :src="src" 
          class="max-w-full max-h-full object-contain transition-transform duration-200 select-none pointer-events-none"
          :style="{
            transform: `translate(${translateX}px, ${translateY}px) scale(${scale})`
          }"
        />
      </div>
      
      <p class="absolute bottom-4 left-1/2 -translate-x-1/2 text-[9px] font-mono text-muted uppercase tracking-[0.2em] opacity-50">Use wheel to zoom, drag to pan</p>
    </div>
  </Transition>
</template>
