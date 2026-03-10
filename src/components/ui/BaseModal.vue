<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-200 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div v-if="props.show" class="fixed inset-0 z-100 flex items-center justify-center p-4 min-h-screen">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-void/60 backdrop-blur-sm" @click="$emit('close')"></div>
        
        <!-- Modal Panel -->
        <div 
          class="relative w-full max-w-md glass-heavy rounded-2xl shadow-2xl overflow-hidden border border-white/10"
          :class="props.panelClass"
        >
          <!-- Header -->
          <div v-if="$slots.header || props.title" class="px-6 py-4 border-b border-white/5 flex items-center justify-between bg-white/2">
            <slot name="header">
              <h3 class="text-lg font-syne font-bold text-text-bright tracking-tight">{{ props.title }}</h3>
            </slot>
            <button 
              @click="$emit('close')"
              class="p-1 rounded-lg text-text-dim hover:text-text-bright hover:bg-white/5 transition-colors"
            >
              <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
              </svg>
            </button>
          </div>

          <!-- Body -->
          <div class="p-6">
            <slot></slot>
          </div>

          <!-- Footer -->
          <div v-if="$slots.footer" class="px-6 py-4 bg-white/2 border-t border-white/5 flex justify-end gap-3">
            <slot name="footer"></slot>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
const props = defineProps<{
  show: boolean;
  title?: string;
  panelClass?: string;
}>();

defineEmits(['close']);
</script>

<style scoped>
.glass-heavy {
  background: rgba(13, 16, 23, 0.8);
  backdrop-filter: blur(24px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.1);
}
</style>
