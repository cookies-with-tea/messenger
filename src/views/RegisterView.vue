<template>
  <AuthLayout>
    <!-- Заголовок — меняется по шагу -->
    <template #title>
      {{ step === 1 ? 'Create account' : 'Check your inbox' }}
    </template>

    <!-- Подзаголовок — меняется по шагу -->
    <template #subtitle>
      <template v-if="step === 1">We'll send a confirmation link to your email</template>
      <template v-else>We've sent a confirmation link to <span class="text-pulse">{{ email }}</span></template>
    </template>

    <!-- Тело — шаг 1 -->
    <form v-if="step === 1" class="flex flex-col gap-4" @submit.prevent="submitEmail">
      <AuthField label="Email" type="email" v-model="email" placeholder="you@example.com" :error="errors.email" />
      <p v-if="globalError" class="text-xs font-mono text-ember text-center -mt-1">{{ globalError }}</p>
      <AuthButton :loading="loading">Send confirmation email</AuthButton>
    </form>

    <!-- Тело — шаг 2 -->
    <div v-else class="flex flex-col items-center gap-5 py-2">
      <div class="relative">
        <div class="w-16 h-16 rounded-2xl bg-pulse/10 border border-pulse/20 flex items-center justify-center text-3xl">
          📬
        </div>
        <div class="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-sage border-2 border-abyss" />
      </div>
      <p class="text-xs font-mono text-text-dim text-center leading-relaxed max-w-xs">
        Click the link in the email to continue.<br/>The link will redirect you to the password setup page.
      </p>
      <button
        @click="resend"
        :disabled="resendCooldown > 0 || loading"
        class="text-xs font-mono text-text-dim hover:text-pulse transition-colors disabled:opacity-50"
      >
        {{ resendCooldown > 0 ? `Resend in ${resendCooldown}s` : 'Resend email' }}
      </button>
    </div>

    <!-- Футер — меняется по шагу -->
    <template #footer>
      <template v-if="step === 1">
        Already have an account?
        <RouterLink to="/login" class="text-pulse hover:text-pulse/80 font-medium transition-colors">Sign in</RouterLink>
      </template>
      <button v-else @click="step = 1; email = ''" class="text-pulse hover:text-pulse/80 font-medium transition-colors">
        ← Use a different email
      </button>
    </template>
  </AuthLayout>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { authApi } from '@/api'
import AuthLayout from '@/components/auth/AuthLayout.vue'
import AuthField  from '@/components/auth/AuthField.vue'
import AuthButton from '@/components/auth/AuthButton.vue'

const step        = ref(1)
const email       = ref('')
const loading     = ref(false)
const globalError = ref('')
const errors      = ref<Record<string, string>>({})
const resendCooldown = ref(0)
let cooldownTimer: ReturnType<typeof setInterval> | null = null

async function submitEmail() {
  errors.value      = {}
  globalError.value = ''
  if (!email.value.trim()) { errors.value.email = 'Required'; return }
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.value)) {
    errors.value.email = 'Invalid email format'
    return
  }

  loading.value = true
  try {
    const res = await authApi.register(email.value.trim())
    if (res.errors || (res.messages && !res.data)) {
      globalError.value = res.messages?.[0] ?? 'Something went wrong'
    } else {
      step.value = 2
      startCooldown()
    }
  } catch {
    globalError.value = 'Network error — try again'
  } finally {
    loading.value = false
  }
}

async function resend() {
  if (resendCooldown.value > 0) return
  loading.value = true
  try {
    await authApi.register(email.value)
    startCooldown()
  } finally {
    loading.value = false
  }
}

function startCooldown() {
  resendCooldown.value = 60
  cooldownTimer = setInterval(() => {
    resendCooldown.value--
    if (resendCooldown.value <= 0 && cooldownTimer) {
      clearInterval(cooldownTimer)
    }
  }, 1000)
}

onUnmounted(() => { if (cooldownTimer) clearInterval(cooldownTimer) })
</script>
