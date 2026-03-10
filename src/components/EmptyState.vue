<template>
  <div class="flex flex-col items-center justify-center h-full gap-10 text-center px-8 bg-void relative overflow-hidden">
    <!-- Sophisticated Background Glows -->
    <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] bg-pulse/5 blur-[120px] rounded-full" />
    <div class="absolute bottom-[-10%] right-[-10%] w-[400px] h-[400px] bg-neon/5 blur-[100px] rounded-full" />

    <!-- Pulsar Orb: Central Visual Element -->
    <div class="relative w-48 h-48 flex items-center justify-center animate-fade-in">
      <!-- Outer Rotating Ring -->
      <svg class="absolute w-full h-full animate-[spinSlow_12s_linear_infinite] opacity-20" viewBox="0 0 100 100">
        <circle cx="50" cy="50" r="48" fill="none" stroke="currentColor" stroke-width="0.5" stroke-dasharray="10 20" class="text-pulse" />
      </svg>
      
      <!-- Middle Counter-Rotating Ring -->
      <svg class="absolute w-[80%] h-[80%] animate-[spinSlow_8s_linear_infinite_reverse] opacity-40" viewBox="0 0 100 100">
        <circle cx="50" cy="50" r="46" fill="none" stroke="currentColor" stroke-width="1" stroke-dasharray="2 15" class="text-pulse-glow" />
      </svg>

      <!-- Inner Steady Ring -->
      <svg class="absolute w-[60%] h-[60%] opacity-30" viewBox="0 0 100 100">
        <circle cx="50" cy="50" r="45" fill="none" stroke="currentColor" stroke-width="0.5" class="text-neon" />
      </svg>

      <!-- Central Glowing Orb -->
      <div class="relative w-16 h-16 rounded-full bg-pulse shadow-[0_0_40px_rgba(var(--color-pulse),0.4)] flex items-center justify-center overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-tr from-white/20 to-transparent" />
        <div class="w-full h-full bg-[radial-gradient(circle_at_30%_30%,rgba(255,255,255,0.2),transparent)]" />
        <!-- Micro-pulsing core -->
        <div class="w-4 h-4 rounded-full bg-white opacity-40 blur-sm animate-pulse" />
      </div>
    </div>

    <!-- Minimalist Typography -->
    <div class="space-y-3 z-10">
      <h2 class="text-2xl font-syne font-extrabold tracking-tight text-text-bright uppercase">Pulsar</h2>
      <p class="text-xs font-mono text-text-dim tracking-[0.2em] uppercase opacity-60">
        Encrypted • Real-time • Minimal
      </p>
    </div>

    <!-- Status Badge: Glassmorphism -->
    <div class="flex items-center gap-2.5 px-5 py-2.5 rounded-2xl bg-white/5 backdrop-blur-md border border-white/10 text-[10px] font-mono text-text-dim uppercase tracking-widest shadow-xl">
      <span class="relative flex h-1.5 w-1.5">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-sage opacity-75"></span>
        <span class="relative inline-flex rounded-full h-1.5 w-1.5 bg-sage"></span>
      </span>
      System Active
    </div>
  </div>
</template>
