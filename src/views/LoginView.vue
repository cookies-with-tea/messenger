<template>
  <AuthLayout>
    <template #title>Sign in</template>
    <template #subtitle>Enter your credentials to continue</template>

    <form class="flex flex-col gap-4" @submit.prevent="submit">
      <AuthField label="Email" type="email" v-model="email" placeholder="you@example.com" :error="errors.email" />
      <AuthField label="Password" type="password" v-model="password" placeholder="••••••••" :error="errors.password" />

      <p v-if="globalError" class="text-xs font-mono text-ember text-center -mt-1">{{ globalError }}</p>

      <AuthButton :loading="loading">Sign in</AuthButton>
    </form>

    <template #footer>
      Don't have an account?
      <RouterLink to="/register" class="text-pulse hover:text-pulse/80 font-medium transition-colors">Create one</RouterLink>
    </template>
  </AuthLayout>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { authApi, tokenStore } from '@/api'
import { useMessengerStore } from '@/stores/messengerStore'
import AuthLayout from '@/components/auth/AuthLayout.vue'
import AuthField  from '@/components/auth/AuthField.vue'
import AuthButton from '@/components/auth/AuthButton.vue'

const router = useRouter()
const route  = useRoute()

const email       = ref('')
const password    = ref('')
const loading     = ref(false)
const globalError = ref('')
const errors      = ref<Record<string, string>>({})

async function submit() {
  errors.value      = {}
  globalError.value = ''

  if (!email.value.trim())    { errors.value.email    = 'Required'; return }
  if (!password.value.trim()) { errors.value.password = 'Required'; return }

  loading.value = true
  try {
    const res = await authApi.login(email.value.trim(), password.value)
    if (res.data) {
      tokenStore.setTokens(res.data)
      // reload store so currentUserId picks up new JWT
      const store = useMessengerStore()
      store.$reset?.()
      const redirect = (route.query.redirect as string) || '/'
      router.push(redirect)
    } else {
      const msg = res.messages?.[0] ?? 'Invalid credentials'
      globalError.value = msg
    }
  } catch {
    globalError.value = 'Network error — try again'
  } finally {
    loading.value = false
  }
}
</script>
