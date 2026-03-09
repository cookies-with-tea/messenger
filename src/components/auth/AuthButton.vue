<template>
  <button
    type="submit"
    :disabled="loading"
    class="w-full py-2.5 rounded-xl text-sm font-syne font-bold transition-all duration-200 flex items-center justify-center gap-2"
    :class="loading
      ? 'bg-pulse/50 text-white/70 cursor-not-allowed'
      : 'bg-pulse text-white hover:bg-pulse/90 shadow-glow active:scale-[0.98]'"
  >
    <span v-if="loading" class="w-4 h-4 rounded-full border-2 border-white/30 border-t-white animate-spin" />
    <slot v-else />
  </button>
</template>

<script setup lang="ts">
defineProps<{ loading: boolean }>()
</script>
