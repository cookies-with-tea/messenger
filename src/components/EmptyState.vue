<template>
  <div class="flex flex-col items-center justify-center h-full gap-6 text-center px-8"
    style="background: radial-gradient(ellipse at center, rgba(79,124,255,0.04) 0%, transparent 70%), #080a0f;">
    <div class="relative">
      <div class="text-7xl animate-bounce-in select-none">👾</div>
      <div class="absolute inset-0 blur-3xl bg-pulse/10 rounded-full" />
    </div>
    <div class="space-y-2">
      <h2 class="text-xl font-syne font-bold text-text-bright">Welcome to Pulsar</h2>
      <p class="text-sm text-text-dim font-mono leading-relaxed max-w-xs">
        Select a conversation from the sidebar to start messaging in real-time.
      </p>
    </div>
    <div class="flex items-center gap-2 px-4 py-2 rounded-full bg-surface border border-border text-xs font-mono text-text-dim">
      <span class="w-2 h-2 rounded-full bg-sage animate-pulse" />
      WebSocket ready
    </div>
  </div>
</template>
