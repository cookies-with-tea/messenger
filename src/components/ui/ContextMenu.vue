<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface MenuItem {
  label: string;
  icon?: string;
  action: () => void;
  variant?: 'default' | 'danger';
}

const props = defineProps<{
  items: MenuItem[];
  x: number;
  y: number;
}>();

const emit = defineEmits(['close']);

const menuRef = ref<HTMLElement | null>(null);

const handleClickOutside = (event: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    emit('close');
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
  // Prevent menu from going off-screen
  if (menuRef.value) {
    const rect = menuRef.value.getBoundingClientRect();
    const screenWidth = window.innerWidth;
    const screenHeight = window.innerHeight;

    if (props.x + rect.width > screenWidth) {
      menuRef.value.style.left = `${screenWidth - rect.width - 10}px`;
    }
    if (props.y + rect.height > screenHeight) {
      menuRef.value.style.top = `${screenHeight - rect.height - 10}px`;
    }
  }
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});
</script>

<template>
  <div
    ref="menuRef"
    class="fixed z-[9999] min-w-[160px] py-1 glass-heavy rounded-xl border border-white/10 shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-100"
    :style="{ left: `${x}px`, top: `${y}px` }"
  >
    <button
      v-for="(item, index) in items"
      :key="index"
      @click="item.action(); emit('close')"
      class="w-full flex items-center px-4 py-2 text-sm transition-colors group relative overflow-hidden"
      :class="item.variant === 'danger' ? 'text-red-400 hover:bg-red-500/20' : 'text-blue-100 hover:bg-white/10'"
    >
      <span v-if="item.icon" class="mr-3 opacity-70 group-hover:opacity-100 transition-opacity" v-html="item.icon"></span>
      <span class="font-medium">{{ item.label }}</span>
      
      <!-- Subtle hover glow -->
      <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-500"></div>
    </button>
  </div>
</template>

<style scoped>
.glass-heavy {
  background: rgba(15, 23, 42, 0.85);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
}
</style>
