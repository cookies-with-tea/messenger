import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([]);

  function add(message: string, type: ToastType = 'info', duration: number = 5000) {
    const id = Math.random().toString(36).substring(2, 9);
    const toast: Toast = { id, message, type, duration };
    toasts.value.push(toast);

    if (duration > 0) {
      setTimeout(() => {
        remove(id);
      }, duration);
    }
  }

  function remove(id: string) {
    const index = toasts.value.findIndex(t => t.id === id);
    if (index !== -1) {
      toasts.value.splice(index, 1);
    }
  }

  function success(msg: string) { add(msg, 'success'); }
  function error(msg: string) { add(msg, 'error', 10000); } 
  function info(msg: string) { add(msg, 'info'); }
  function warning(msg: string) { add(msg, 'warning'); }

  return {
    toasts,
    add,
    remove,
    success,
    error,
    info,
    warning
  };
});
