<template>
  <AuthLayout>
    <!-- Заголовок -->
    <template #title>
      <template v-if="step === 'verifying'">Verifying...</template>
      <template v-else-if="step === 'invalid'">Link expired</template>
      <template v-else-if="step === 'form'">Set up your account</template>
      <template v-else>You're in! 🎉</template>
    </template>

    <!-- Подзаголовок -->
    <template #subtitle>
      <template v-if="step === 'verifying'">Please wait a moment</template>
      <template v-else-if="step === 'invalid'">This confirmation link is invalid or has already been used</template>
      <template v-else-if="step === 'form'">Almost there — fill in your details</template>
      <template v-else>Account created successfully</template>
    </template>

    <!-- Verifying -->
    <div v-if="step === 'verifying'" class="flex justify-center py-6">
      <div class="w-10 h-10 rounded-full border-2 border-pulse border-t-transparent animate-spin" />
    </div>

    <!-- Invalid / expired -->
    <div v-else-if="step === 'invalid'" class="flex flex-col items-center gap-4 py-2">
      <div class="text-4xl">🔗</div>
      <RouterLink to="/register">
        <AuthButton :loading="false">Request a new link</AuthButton>
      </RouterLink>
    </div>

    <!-- Create password form -->
    <form v-else-if="step === 'form'" class="flex flex-col gap-4" @submit.prevent="submitCreate">
      <div class="grid grid-cols-2 gap-3">
        <AuthField label="First name" v-model="firstName" placeholder="Ivan" :error="errors.first_name" />
        <AuthField label="Last name"  v-model="lastName"  placeholder="Petrov" :error="errors.last_name" />
      </div>
      <AuthField label="Email" type="email" v-model="confirmedEmail" placeholder="you@example.com" :error="errors.email" />
      <AuthField label="Password" type="password" v-model="password" placeholder="min. 8 characters" :error="errors.password" />
      <AuthField label="Confirm password" type="password" v-model="passwordConfirm" placeholder="••••••••" :error="errors.password_confirm" />
      <p v-if="globalError" class="text-xs font-mono text-ember text-center -mt-1">{{ globalError }}</p>
      <AuthButton :loading="loading">Create account</AuthButton>
    </form>

    <!-- Success -->
    <div v-else class="flex flex-col items-center gap-4 py-2">
      <div class="w-16 h-16 rounded-2xl bg-sage/10 border border-sage/20 flex items-center justify-center text-3xl">✅</div>
      <RouterLink to="/login">
        <AuthButton :loading="false">Go to sign in</AuthButton>
      </RouterLink>
    </div>

    <!-- Футер -->
    <template #footer>
      <template v-if="step === 'form' || step === 'done'">
        Already registered?
        <RouterLink to="/login" class="text-pulse hover:text-pulse/80 font-medium transition-colors">Sign in</RouterLink>
      </template>
    </template>
  </AuthLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { authApi } from '@/api'
import AuthLayout from '@/components/auth/AuthLayout.vue'
import AuthField  from '@/components/auth/AuthField.vue'
import AuthButton from '@/components/auth/AuthButton.vue'

const route = useRoute()

type Step = 'verifying' | 'invalid' | 'form' | 'done'
const step            = ref<Step>('verifying')
const confirmedEmail  = ref('')
const firstName       = ref('')
const lastName        = ref('')
const password        = ref('')
const passwordConfirm = ref('')
const loading         = ref(false)
const globalError     = ref('')
const errors          = ref<Record<string, string>>({})
const confirmKey      = ref('')

onMounted(async () => {
  const key = route.query.key as string
  if (!key) { step.value = 'invalid'; return }
  confirmKey.value = key

  try {
    const res = await authApi.checkKey(key)
    // Backend returns the email in data or messages — adjust if needed
    if (res.errors && Object.keys(res.errors).length > 0) {
      step.value = 'invalid'
    } else {
      // If backend returns email in data, pre-fill it
      step.value = 'form'
    }
  } catch {
    step.value = 'invalid'
  }
})

async function submitCreate() {
  errors.value      = {}
  globalError.value = ''

  if (!confirmedEmail.value.trim())  { errors.value.email    = 'Required'; return }
  if (!password.value)               { errors.value.password = 'Required'; return }
  if (password.value.length < 8)     { errors.value.password = 'Min. 8 characters'; return }
  if (password.value !== passwordConfirm.value) {
    errors.value.password_confirm = 'Passwords don\'t match'
    return
  }

  loading.value = true
  try {
    const res = await authApi.createUser({
      email:      confirmedEmail.value.trim(),
      password:   password.value,
      first_name: firstName.value.trim() || undefined,
      last_name:  lastName.value.trim()  || undefined,
    })
    if (res.errors && Object.keys(res.errors).length > 0) {
      // Map backend field errors
      Object.entries(res.errors).forEach(([k, v]) => {
        errors.value[k] = Array.isArray(v) ? v[0] : String(v)
      })
      globalError.value = res.messages?.[0] ?? ''
    } else {
      step.value = 'done'
    }
  } catch {
    globalError.value = 'Network error — try again'
  } finally {
    loading.value = false
  }
}
</script>
