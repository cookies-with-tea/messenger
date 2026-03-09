<template>
  <div class="flex flex-col gap-1.5">
    <label class="text-xs font-mono font-medium text-text-dim tracking-wide uppercase">{{ label }}</label>
    <div class="relative">
      <input
        :type="showPassword ? 'text' : type"
        :value="modelValue"
        :placeholder="placeholder"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        class="w-full bg-surface border rounded-xl px-4 py-2.5 text-sm text-text-bright placeholder:text-muted outline-none transition-all font-mono"
        :class="error
          ? 'border-ember/60 focus:border-ember'
          : 'border-border focus:border-pulse/60 focus:bg-elevated'"
        :autocomplete="type === 'password' ? 'current-password' : 'email'"
      />
      <!-- Toggle password -->
      <button
        v-if="type === 'password'"
        type="button"
        @click="showPassword = !showPassword"
        class="absolute right-3 top-1/2 -translate-y-1/2 text-muted hover:text-text-dim transition-colors"
        tabindex="-1"
      >
        <svg v-if="!showPassword" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10 12a2 2 0 100-4 2 2 0 000 4z"/>
          <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd"/>
        </svg>
        <svg v-else class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M3.28 2.22a.75.75 0 00-1.06 1.06l14.5 14.5a.75.75 0 101.06-1.06l-1.745-1.745a10.029 10.029 0 003.3-4.38 1.651 1.651 0 000-1.185A10.004 10.004 0 009.999 3a9.956 9.956 0 00-4.744 1.194L3.28 2.22zM7.752 6.69l1.092 1.092a2.5 2.5 0 013.374 3.373l1.091 1.092a4 4 0 00-5.557-5.557z" clip-rule="evenodd"/>
          <path d="M10.748 13.93l2.523 2.523a9.987 9.987 0 01-3.27.547c-4.258 0-7.894-2.66-9.337-6.41a1.651 1.651 0 010-1.186A10.007 10.007 0 012.839 6.02L6.07 9.252a4 4 0 004.678 4.678z"/>
        </svg>
      </button>
    </div>
    <p v-if="error" class="text-[11px] font-mono text-ember">{{ error }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
defineProps<{ label: string; modelValue: string; type?: string; placeholder?: string; error?: string }>()
defineEmits<{ 'update:modelValue': [value: string] }>()
const showPassword = ref(false)
</script>
