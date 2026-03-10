<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="open"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="close"
      >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="close" />

        <!-- Panel -->
        <div class="relative w-full max-w-md bg-abyss border border-border rounded-2xl shadow-2xl flex flex-col overflow-hidden">

          <!-- Header -->
          <div class="flex items-center justify-between px-5 py-4 border-b border-border">
            <h2 class="text-sm font-syne font-bold text-text-bright">New Chat</h2>
            <button @click="close" class="p-1.5 rounded-lg text-text-dim hover:text-text-bright hover:bg-elevated transition-all">
              <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z"/>
              </svg>
            </button>
          </div>

          <!-- Tabs -->
          <div class="flex border-b border-border">
            <button
              v-for="tab in tabs" :key="tab.id"
              @click="activeTab = tab.id"
              class="flex-1 py-2.5 text-xs font-mono font-medium transition-all"
              :class="activeTab === tab.id
                ? 'text-pulse border-b-2 border-pulse'
                : 'text-text-dim hover:text-text-base'"
            >{{ tab.label }}</button>
          </div>

          <!-- Direct chat tab -->
          <div v-if="activeTab === 'direct'" class="flex flex-col p-4 gap-3">
            <!-- Search -->
            <div class="relative">
              <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd"/>
              </svg>
              <input
                v-model="searchQuery"
                @input="onSearch"
                type="text"
                placeholder="Search by name or email..."
                class="w-full bg-surface border border-border rounded-xl py-2.5 pl-9 pr-4 text-sm text-text-bright placeholder:text-muted outline-none focus:border-pulse/60 transition-all font-mono"
                autofocus
              />
            </div>

            <!-- Results -->
            <div class="max-h-64 overflow-y-auto space-y-1 scrollbar-thin -mx-1 px-1">
              <div v-if="searching" class="flex justify-center py-6">
                <span class="text-xs font-mono text-muted animate-pulse">Searching...</span>
              </div>
              <div v-else-if="users.length === 0 && searchQuery" class="flex flex-col items-center gap-2 py-6 text-muted">
                <span class="text-xl">🔍</span>
                <span class="text-xs font-mono">No users found</span>
              </div>
              <div v-else-if="users.length === 0" class="flex flex-col items-center gap-2 py-6 text-muted">
                <span class="text-xl">👥</span>
                <span class="text-xs font-mono">Start typing to search</span>
              </div>
              <button
                v-for="user in users"
                :key="user.uuid"
                @click="startDirectChat(user)"
                :disabled="creating"
                class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-elevated border border-transparent hover:border-border transition-all text-left"
              >
                <div
                  class="w-9 h-9 rounded-full bg-elevated border border-border flex items-center justify-center text-sm font-bold shrink-0"
                  :style="{ color: colorFor(user.uuid) }"
                >
                  {{ initials(user) }}
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium text-text-bright truncate">{{ fullName(user) }}</p>
                  <p class="text-xs font-mono text-text-dim truncate">{{ user.email }}</p>
                </div>
                <svg class="w-4 h-4 text-text-dim shrink-0" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M2 10a.75.75 0 01.75-.75h12.59l-2.1-1.95a.75.75 0 111.02-1.1l3.5 3.25a.75.75 0 010 1.1l-3.5 3.25a.75.75 0 11-1.02-1.1l2.1-1.95H2.75A.75.75 0 012 10z" clip-rule="evenodd"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- Group chat tab -->
          <div v-else class="flex flex-col p-4 gap-3">
            <input
              v-model="groupName"
              type="text"
              placeholder="Group name..."
              maxlength="80"
              class="w-full bg-surface border border-border rounded-xl py-2.5 px-4 text-sm text-text-bright placeholder:text-muted outline-none focus:border-pulse/60 transition-all font-mono"
            />

            <!-- Search members -->
            <div class="relative">
              <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd"/>
              </svg>
              <input
                v-model="searchQuery"
                @input="onSearch"
                type="text"
                placeholder="Add members..."
                class="w-full bg-surface border border-border rounded-xl py-2.5 pl-9 pr-4 text-sm text-text-bright placeholder:text-muted outline-none focus:border-pulse/60 transition-all font-mono"
              />
            </div>

            <!-- Selected members -->
            <div v-if="selectedUsers.length > 0" class="flex flex-wrap gap-2">
              <div
                v-for="user in selectedUsers"
                :key="user.uuid"
                class="flex items-center gap-1.5 pl-2 pr-1 py-1 rounded-full bg-pulse/20 border border-pulse/30 text-xs font-mono text-pulse"
              >
                <span>{{ fullName(user) }}</span>
                <button @click="removeSelected(user)" class="w-4 h-4 rounded-full hover:bg-pulse/30 flex items-center justify-center transition-all">
                  <svg class="w-2.5 h-2.5" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z"/>
                  </svg>
                </button>
              </div>
            </div>

            <!-- User list -->
            <div class="max-h-40 overflow-y-auto space-y-1 scrollbar-thin -mx-1 px-1">
              <div v-if="searching" class="text-center py-4">
                <span class="text-xs font-mono text-muted animate-pulse">Searching...</span>
              </div>
              <button
                v-for="user in filteredUsers"
                :key="user.uuid"
                @click="toggleSelect(user)"
                class="w-full flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-elevated border border-transparent hover:border-border transition-all text-left"
              >
                <div
                  class="w-8 h-8 rounded-full bg-elevated border border-border flex items-center justify-center text-xs font-bold shrink-0"
                  :style="{ color: colorFor(user.uuid) }"
                >{{ initials(user) }}</div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium text-text-bright truncate">{{ fullName(user) }}</p>
                  <p class="text-xs font-mono text-text-dim truncate">{{ user.email }}</p>
                </div>
                <div
                  class="w-4 h-4 rounded border flex items-center justify-center transition-all"
                  :class="isSelected(user) ? 'bg-pulse border-pulse' : 'border-border'"
                >
                  <svg v-if="isSelected(user)" class="w-2.5 h-2.5 text-white" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                </div>
              </button>
            </div>

            <!-- Create button -->
            <button
              @click="createGroup"
              :disabled="!groupName.trim() || selectedUsers.length === 0 || creating"
              class="mt-1 py-2.5 rounded-xl text-sm font-syne font-bold transition-all"
              :class="groupName.trim() && selectedUsers.length > 0 && !creating
                ? 'bg-pulse text-white hover:bg-pulse/90 shadow-glow'
                : 'bg-surface text-muted border border-border cursor-not-allowed'"
            >
              {{ creating ? 'Creating...' : `Create group · ${selectedUsers.length} member${selectedUsers.length !== 1 ? 's' : ''}` }}
            </button>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useMessengerStore } from '@/stores/messengerStore'
import { userApi, chatApi, type UserResponseDTO } from '@/api'

const props  = defineProps<{ open: boolean }>()
const emit   = defineEmits<{ close: [] }>()
const router = useRouter()

const store   = useMessengerStore()
const activeTab  = ref<'direct' | 'group'>('direct')
const tabs = [
  { id: 'direct' as const, label: 'Direct message' },
  { id: 'group'  as const, label: 'Group chat' },
]

const searchQuery   = ref('')
const users         = ref<UserResponseDTO[]>([])
const searching     = ref(false)
const creating      = ref(false)
const groupName     = ref('')
const selectedUsers = ref<UserResponseDTO[]>([])

let searchTimer: ReturnType<typeof setTimeout> | null = null

const COLOR_PALETTE = ['#4f7cff','#00e5ff','#52e07c','#ff6b35','#c084fc','#fb923c','#38bdf8','#f472b6']
function colorFor(uuid: string) {
  return COLOR_PALETTE[uuid.charCodeAt(0) % COLOR_PALETTE.length]
}
function initials(u: UserResponseDTO) {
  return [u.first_name, u.last_name].filter(Boolean).map(s => s![0]).join('').toUpperCase() || '?'
}
function fullName(u: UserResponseDTO) {
  return [u.first_name, u.last_name].filter(Boolean).join(' ') || u.email || u.uuid
}

function onSearch() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 300)
}

async function doSearch() {
  if (!searchQuery.value.trim()) { users.value = []; return }
  searching.value = true
  try {
    const res = await userApi.search(searchQuery.value)
    users.value = (res.data?.items ?? []).filter(u => u.uuid !== store.currentUserId)
  } finally {
    searching.value = false
  }
}

const filteredUsers = computed(() =>
  users.value.filter(u => !isSelected(u))
)

function isSelected(u: UserResponseDTO) {
  return selectedUsers.value.some(s => s.uuid === u.uuid)
}
function toggleSelect(u: UserResponseDTO) {
  isSelected(u) ? removeSelected(u) : selectedUsers.value.push(u)
}
function removeSelected(u: UserResponseDTO) {
  selectedUsers.value = selectedUsers.value.filter(s => s.uuid !== u.uuid)
}

async function startDirectChat(user: UserResponseDTO) {
  creating.value = true
  try {
    const res = await chatApi.create({
      chat_type: 'direct',
      member_uuids: [user.uuid],
    })
    if (res.data) {
      await store.fetchChats()
      await store.selectChat(res.data.uuid)
      if (router.currentRoute.value.path !== `/chat/${res.data.uuid}`) {
        router.push(`/chat/${res.data.uuid}`)
      }
      close()
    }
  } finally {
    creating.value = false
  }
}

async function createGroup() {
  if (!groupName.value.trim() || selectedUsers.value.length === 0) return
  creating.value = true
  try {
    const res = await chatApi.create({
      chat_type: 'group',
      name: groupName.value.trim(),
      member_uuids: selectedUsers.value.map(u => u.uuid),
    })
    if (res.data) {
      await store.fetchChats()
      await store.selectChat(res.data.uuid)
      if (router.currentRoute.value.path !== `/chat/${res.data.uuid}`) {
        router.push(`/chat/${res.data.uuid}`)
      }
      close()
    }
  } finally {
    creating.value = false
  }
}

function close() {
  emit('close')
}

// Reset on close
watch(() => props.open, (v) => {
  if (!v) {
    searchQuery.value = ''
    users.value = []
    groupName.value = ''
    selectedUsers.value = []
    activeTab.value = 'direct'
  }
})
</script>

<style scoped>
.modal-enter-active, .modal-leave-active {
  transition: opacity 0.2s;
}
.modal-enter-from, .modal-leave-to {
  opacity: 0;
}
.modal-enter-active .relative,
.modal-leave-active .relative {
  transition: transform 0.2s, opacity 0.2s;
}
.modal-enter-from .relative,
.modal-leave-to .relative {
  transform: scale(0.95) translateY(-8px);
  opacity: 0;
}
</style>
