<template>
  <div class="toast-container">
    <TransitionGroup name="toast-list">
      <div 
        v-for="toast in store.toasts" 
        :key="toast.id" 
        class="toast-item glass"
        :class="toast.type"
      >
        <div class="toast-icon">
          <i v-if="toast.type === 'success'" class="fas fa-check-circle"></i>
          <i v-else-if="toast.type === 'error'" class="fas fa-exclamation-triangle"></i>
          <i v-else-if="toast.type === 'warning'" class="fas fa-exclamation-circle"></i>
          <i v-else class="fas fa-info-circle"></i>
        </div>
        <div class="toast-message">{{ toast.message }}</div>
        <button class="toast-close" @click="store.remove(toast.id)">
          <i class="fas fa-times"></i>
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { useToastStore } from '@/stores/toastStore';

const store = useToastStore();
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 24px;
  right: 24px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 12px;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  min-width: 300px;
  max-width: 450px;
  padding: 16px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(20px);
}

.toast-icon {
  font-size: 1.25rem;
  flex-shrink: 0;
}

.toast-message {
  flex-grow: 1;
  font-size: 0.9rem;
  line-height: 1.4;
}

.toast-close {
  background: transparent;
  border: none;
  color: white;
  opacity: 0.5;
  cursor: pointer;
  padding: 4px;
  transition: opacity 0.2s;
}

.toast-close:hover {
  opacity: 1;
}

/* Colors */
.success {
  border-left: 4px solid #52e07c;
}
.success .toast-icon { color: #52e07c; }

.error {
  border-left: 4px solid #ff4f4f;
  background: rgba(255, 79, 79, 0.05);
}
.error .toast-icon { color: #ff4f4f; }

.warning {
  border-left: 4px solid #ffab00;
}
.warning .toast-icon { color: #ffab00; }

.info {
  border-left: 4px solid var(--accent-color, #4f7cff);
}
.info .toast-icon { color: var(--accent-color, #4f7cff); }

/* Animations */
.toast-list-enter-active,
.toast-list-leave-active {
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.toast-list-enter-from {
  opacity: 0;
  transform: translateX(30px) scale(0.9);
}

.toast-list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
