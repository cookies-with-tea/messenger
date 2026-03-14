<template>
  <div 
    class="app-image relative overflow-hidden flex items-center justify-center shrink-0 shadow-sm"
    :class="[
      className,
      !isLoaded && !hasError ? 'animate-pulse bg-white/5' : '',
      hasError ? 'bg-linear-to-br from-pulse/20 to-pulse/5 border border-pulse/30' : ''
    ]"
    :style="containerStyle"
  >
    <!-- Placeholder while loading or error -->
    <div v-if="hasError" class="flex items-center justify-center w-full h-full text-pulse font-bold uppercase tracking-wider select-none">
      <slot name="fallback">
        {{ initials || '?' }}
      </slot>
    </div>

    <img
      v-if="src && !hasError"
      ref="imgRef"
      :src="src"
      :alt="alt"
      :srcset="srcset"
      class="w-full h-full object-cover transition-opacity duration-500 ease-in-out"
      :class="isLoaded ? 'opacity-100' : 'opacity-0'"
      @load="onLoad"
      @error="onError"
    />

    <!-- Skeleton overlay while loading -->
    <div 
      v-if="!isLoaded && !hasError" 
      class="absolute inset-0 bg-linear-to-r from-transparent via-white/5 to-transparent skeleton-shimmer"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

const props = defineProps<{
  src?: string | null
  alt?: string
  initials?: string
  size?: number | string
  className?: string
  // For webp/2x support manually if backend doesnt provide
  src2x?: string
  srcWebp?: string
}>()

const isLoaded = ref(false)
const hasError = ref(false)
const imgRef = ref<HTMLImageElement | null>(null)

const containerStyle = computed(() => {
  if (!props.size) return {}
  const s = typeof props.size === 'number' ? `${props.size}px` : props.size
  return {
    width: s,
    height: s,
    fontSize: typeof props.size === 'number' ? `${props.size * 0.4}px` : 'inherit'
  }
})

const srcset = computed(() => {
  if (props.src2x && props.src) {
    return `${props.src} 1x, ${props.src2x} 2x`
  }
  return undefined
})

watch(() => props.src, () => {
  isLoaded.value = false
  hasError.value = false
})

function onLoad() {
  isLoaded.value = true
}

function onError() {
  hasError.value = true
  isLoaded.value = false
}
</script>

<style scoped>
.skeleton-shimmer {
  background-size: 200% 100%;
  animation: shimmer 2s infinite linear;
}

@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}
</style>
