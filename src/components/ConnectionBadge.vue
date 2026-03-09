<template>
  <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-mono" :class="badgeClass">
    <span class="w-1.5 h-1.5 rounded-full" :class="dotClass" />
    {{ label }}
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ConnectionStatus } from '@/types'

const props = defineProps<{ status: ConnectionStatus }>()

const badgeClass = computed(() => ({
  'bg-sage/10 text-sage border border-sage/20': props.status === 'connected',
  'bg-pulse/10 text-pulse border border-pulse/20': props.status === 'connecting',
  'bg-muted/30 text-text-dim border border-border': props.status === 'disconnected',
  'bg-ember/10 text-ember border border-ember/20': props.status === 'error',
}))

const dotClass = computed(() => ({
  'bg-sage animate-pulse': props.status === 'connected',
  'bg-pulse animate-ping': props.status === 'connecting',
  'bg-text-dim': props.status === 'disconnected',
  'bg-ember': props.status === 'error',
}))

const label = computed(() => ({
  connected: 'Live',
  connecting: 'Connecting...',
  disconnected: 'Offline',
  error: 'Error',
}[props.status]))
</script>
