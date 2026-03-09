<template>
  <div class="auth-layout min-h-screen bg-void flex items-center justify-center p-4 relative overflow-hidden">
    <!-- Background glow -->
    <div class="absolute inset-0 pointer-events-none">
      <div class="absolute top-1/4 left-1/2 -translate-x-1/2 w-[600px] h-[400px] bg-pulse/5 rounded-full blur-3xl" />
      <div class="absolute bottom-1/4 right-1/3 w-[300px] h-[300px] bg-neon/3 rounded-full blur-3xl" />
    </div>

    <!-- Grid pattern -->
    <div
      class="absolute inset-0 opacity-[0.02] pointer-events-none"
      style="background-image: linear-gradient(rgba(79,124,255,0.5) 1px, transparent 1px), linear-gradient(90deg, rgba(79,124,255,0.5) 1px, transparent 1px); background-size: 40px 40px;"
    />

    <!-- Card -->
    <div class="relative w-full max-w-sm animate-slide-up">
      <!-- Logo -->
      <div class="flex items-center gap-2 mb-8 justify-center">
        <span class="text-2xl font-syne font-extrabold text-text-bright tracking-tight">pulsar</span>
        <span class="text-pulse text-2xl leading-none">·</span>
      </div>

      <div class="bg-abyss/80 backdrop-blur-xl border border-border rounded-2xl p-6 shadow-2xl">
        <!-- Title area -->
        <div class="mb-6 space-y-1">
          <h1 class="text-xl font-syne font-bold text-text-bright">
            <slot name="title" />
          </h1>
          <p class="text-sm font-mono text-text-dim">
            <slot name="subtitle" />
          </p>
        </div>

        <!-- Form slot -->
        <slot />

        <!-- Footer -->
        <p class="mt-5 text-center text-xs font-mono text-text-dim">
          <slot name="footer" />
        </p>
      </div>
    </div>
  </div>
</template>
