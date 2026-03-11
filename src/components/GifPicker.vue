<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'

const props = defineProps<{
  initialMode?: 'gif' | 'sticker'
}>()

const emit = defineEmits<{
  select: [url: string]
  close: []
}>()

const apiKey = import.meta.env.VITE_GIF_API_KEY // Giphy Public Beta Key
const mode = ref<'gif' | 'sticker'>(props.initialMode || 'gif')
const items = ref<any[]>([])
const searchQuery = ref('')
const loading = ref(false)
const error = ref(false)

async function fetchItems(query = '') {
  loading.value = true
  error.value = false
  const type = mode.value === 'sticker' ? 'stickers' : 'gifs'
  const endpoint = query 
    ? `https://api.giphy.com/v1/${type}/search?api_key=${apiKey}&q=${encodeURIComponent(query)}&limit=20`
    : `https://api.giphy.com/v1/${type}/trending?api_key=${apiKey}&limit=20`

  try {
    const res = await fetch(endpoint)
    const data = await res.json()
    if (data.data) {
      items.value = data.data
    } else {
      error.value = true
    }
  } catch (e) {
    error.value = true
  } finally {
    loading.value = false
  }
}

let timeout: ReturnType<typeof setTimeout>
watch(searchQuery, (q) => {
  clearTimeout(timeout)
  timeout = setTimeout(() => {
    fetchItems(q)
  }, 500)
})

watch(mode, () => {
  fetchItems(searchQuery.value)
})

onMounted(() => {
  fetchItems()
})

function selectItem(item: any) {
  const url = item.images.fixed_height.url
  emit('select', `![${mode.value}](${url})`)
}
</script>

<template>
  <div class="flex flex-col w-[300px] sm:w-[350px] h-[400px] glass-heavy border border-white/10 rounded-2xl shadow-2xl backdrop-blur-3xl overflow-hidden animate-in fade-in zoom-in-95 duration-200">
    <!-- Tabs -->
    <div class="flex border-b border-white/5 bg-white/5">
      <button 
        @click="mode = 'gif'" 
        class="flex-1 py-2 text-[10px] font-black uppercase tracking-widest transition-colors"
        :class="mode === 'gif' ? 'text-pulse bg-white/5 border-b-2 border-pulse' : 'text-text-dim hover:text-text-bright hover:bg-white/5'"
      >GIFs</button>
      <button 
        @click="mode = 'sticker'" 
        class="flex-1 py-2 text-[10px] font-black uppercase tracking-widest transition-colors"
        :class="mode === 'sticker' ? 'text-ember bg-white/5 border-b-2 border-ember' : 'text-text-dim hover:text-text-bright hover:bg-white/5'"
      >Stickers</button>
    </div>

    <!-- Header -->
    <div class="p-3 border-b border-white/5 flex items-center gap-2">
      <div class="flex-1 relative">
        <input 
          v-model="searchQuery"
          type="text" 
          placeholder="Search Giphy..."
          class="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-2 text-xs font-mono text-text-bright placeholder:text-muted outline-none focus:border-pulse/40 transition-all"
        />
        <div v-if="loading" class="absolute right-3 top-1/2 -translate-y-1/2">
          <div class="w-3 h-3 border-2 border-pulse border-t-transparent rounded-full animate-spin"></div>
        </div>
      </div>
      <button @click="$emit('close')" class="shrink-0 p-2 hover:bg-white/10 rounded-lg text-text-dim transition-colors">
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>

    <!-- Grid -->
    <div class="flex-1 overflow-y-auto p-3 scrollbar-thin">
      <div v-if="error" class="h-full flex flex-col items-center justify-center text-center p-6 gap-3">
        <span class="text-2xl">📡</span>
        <span class="text-[10px] font-mono font-black uppercase tracking-widest text-ember">Signal Interrupted</span>
        <button @click="fetchItems(searchQuery)" class="text-[9px] font-mono text-pulse hover:underline">Retry Connection</button>
      </div>

      <div v-else-if="items.length === 0 && !loading" class="h-full flex flex-col items-center justify-center text-center p-6 gap-2">
        <span class="text-2xl">🌫️</span>
        <span class="text-[10px] font-mono font-black uppercase tracking-widest text-muted">Nothing Found</span>
      </div>

      <div v-else class="grid grid-cols-2 gap-2">
        <div 
          v-for="item in items" 
          :key="item.id"
          @click="selectItem(item)"
          class="aspect-square relative group cursor-pointer overflow-hidden rounded-xl bg-white/5 border border-white/5 hover:border-pulse/40 transition-all flex items-center justify-center"
        >
          <img 
            :src="item.images.fixed_height_small.url" 
            class="w-full h-full object-cover transition-transform group-hover:scale-110" 
            :class="{ 'object-contain p-2': mode === 'sticker' }"
            loading="lazy"
          />
          <div class="absolute inset-0 bg-pulse/20 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
            <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="p-2 border-t border-white/5 flex justify-center">
       <img src="https://giphy.com/static/img/powered-by-giphy.png" class="h-4 opacity-50 contrast-0" alt="Giphy" />
    </div>
  </div>
</template>
