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

    <!-- Тело — шаг 1 (Email) -->
    <template v-if="step === 1">
      <form v-if="IS_REGISTRATION_EMAIL" class="flex flex-col gap-4" @submit.prevent="submitEmail">
        <AuthField label="Email" type="email" v-model="email" placeholder="you@example.com" :error="errors.email" />
        <p v-if="globalError" class="text-xs font-mono text-ember text-center -mt-1">{{ globalError }}</p>
        <AuthButton :loading="loading">Send confirmation email</AuthButton>
      </form>

      <!-- Прямая регистрация -->
      <form v-else class="flex flex-col gap-4" @submit.prevent="submitDirect">
        <div class="grid grid-cols-2 gap-3">
          <AuthField label="First name" v-model="firstName" placeholder="Ivan" :error="errors.first_name" />
          <AuthField label="Last name"  v-model="lastName"  placeholder="Petrov" :error="errors.last_name" />
        </div>
        <AuthField label="Email" type="email" v-model="email" placeholder="you@example.com" :error="errors.email" />
        <AuthField label="Password" type="password" v-model="password" placeholder="min. 8 characters" :error="errors.password" />
        <AuthField label="Confirm password" type="password" v-model="passwordConfirm" placeholder="••••••••" :error="errors.password_confirm" />
        <p v-if="globalError" class="text-xs font-mono text-ember text-center -mt-1">{{ globalError }}</p>
        <AuthButton :loading="loading">Create account</AuthButton>
      </form>
    </template>

    <!-- Тело — шаг 2 (Email confirmation sent) -->
    <div v-else-if="step === 2" class="flex flex-col items-center gap-5 py-2">
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

    <!-- Тело — успех прямой регистрации -->
    <div v-else class="flex flex-col items-center gap-4 py-2">
      <div class="w-16 h-16 rounded-2xl bg-sage/10 border border-sage/20 flex items-center justify-center text-3xl">✅</div>
      <p class="text-xs font-mono text-text-dim text-center">Account created successfully</p>
      <RouterLink to="/login" class="w-full">
        <AuthButton :loading="false">Go to sign in</AuthButton>
      </RouterLink>
    </div>

    <!-- Футер — меняется по шагу -->
    <template #footer>
      <template v-if="step === 1 || step === 3">
        Already have an account?
        <RouterLink to="/login" class="text-pulse hover:text-pulse/80 font-medium transition-colors">Sign in</RouterLink>
      </template>
      <button v-else-if="step === 2" @click="step = 1; email = ''" class="text-pulse hover:text-pulse/80 font-medium transition-colors">
        ← Use a different email
      </button>
    </template>
  </AuthLayout>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { authApi } from '@/api'
import { IS_REGISTRATION_EMAIL } from '@/config'
import AuthLayout from '@/components/auth/AuthLayout.vue'
import AuthField  from '@/components/auth/AuthField.vue'
import AuthButton from '@/components/auth/AuthButton.vue'

const step            = ref(1)
const email           = ref('')
const firstName       = ref('')
const lastName        = ref('')
const password        = ref('')
const passwordConfirm = ref('')
const loading         = ref(false)
const globalError     = ref('')
const errors          = ref<Record<string, string>>({})
const resendCooldown  = ref(0)
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
    const res = await authApi.register({ email: email.value.trim() })
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

async function submitDirect() {
  errors.value      = {}
  globalError.value = ''

  if (!email.value.trim())           { errors.value.email            = 'Required'; return }
  if (!password.value)             { errors.value.password         = 'Required'; return }
  if (password.value.length < 8)   { errors.value.password         = 'Min. 8 characters'; return }
  if (password.value !== passwordConfirm.value) {
    errors.value.password_confirm = 'Passwords don\'t match'
    return
  }

  loading.value = true
  try {
    const res = await authApi.register({
      email:      email.value.trim(),
      password:   password.value,
      first_name: firstName.value.trim() || undefined,
      last_name:  lastName.value.trim()  || undefined,
    })
    
    if (res.errors && Object.keys(res.errors).length > 0) {
      Object.entries(res.errors).forEach(([k, v]) => {
        errors.value[k] = Array.isArray(v) ? v[0] : String(v)
      })
      globalError.value = res.messages?.[0] ?? ''
    } else {
      step.value = 3 // Success step for direct registration
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
    await authApi.register({ email: email.value })
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
