<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import VoiceRecorder from './VoiceRecorder.vue'
import GifPicker from './GifPicker.vue'
import ContextMenu from './ui/ContextMenu.vue'

const store = useMessengerStore()

const EMOJIS = ['😀','😂','🥹','😎','🤔','🚀','💡','🔥','❤️','👍','👎','🎉','⚡','🌊','🎮','🎨','💻','🤖','👾','⭐','✨','🎯','📱','🏆']

const emit = defineEmits<{
  send: [text: string]
  typing: []
  stopTyping: []
}>()

const text = ref('')
const showEmoji = ref(false)
const showGif = ref(false)
const pickerMode = ref<'gif' | 'sticker'>('gif')
const inputRef = ref<HTMLTextAreaElement>()
const fileInput = ref<HTMLInputElement>()

// Formatting Menu State
const showFormattingMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)
const selectedText = ref('')
const selectionStart = ref(0)
const selectionEnd = ref(0)

function handleInputContextMenu(e: MouseEvent) {
  const el = inputRef.value
  if (!el) return

  const start = el.selectionStart
  const end = el.selectionEnd
  
  if (start !== end) {
    e.preventDefault()
    selectionStart.value = start
    selectionEnd.value = end
    selectedText.value = text.value.substring(start, end)
    menuX.value = e.clientX
    menuY.value = e.clientY
    showFormattingMenu.value = true
  }
}

function formatText(type: 'bold' | 'italic' | 'code' | 'monospace') {
  const start = selectionStart.value
  const end = selectionEnd.value
  const prefix = text.value.substring(0, start)
  const mid = selectedText.value
  const suffix = text.value.substring(end)

  let formatted = mid
  switch (type) {
    case 'bold': formatted = `**${mid}**`; break
    case 'italic': formatted = `_${mid}_`; break
    case 'code': formatted = `\n\`\`\`\n${mid}\n\`\`\`\n`; break
    case 'monospace': formatted = `\`${mid}\``; break
  }

  text.value = prefix + formatted + suffix
  showFormattingMenu.value = false
  
  // Refocus and place cursor
  nextTick(() => {
    inputRef.value?.focus()
    const newPos = start + formatted.length
    inputRef.value?.setSelectionRange(newPos, newPos)
  })
}

const formattingItems = computed(() => [
  { label: 'Bold', action: () => formatText('bold') },
  { label: 'Italic', action: () => formatText('italic') },
  { label: 'Monospace', action: () => formatText('monospace') },
  { label: 'Code Block', action: () => formatText('code') },
])

let typingTimer: ReturnType<typeof setTimeout> | null = null
let isTypingActive = false

const textareaHeight = computed(() => {
  const lines = (text.value.match(/\n/g) ?? []).length + 1
  return `${Math.min(lines * 24 + 24, 120)}px`
})

const replySender = computed(() => {
  const msg = store.replyingToMessage
  if (!msg) return ''
  return msg.sender?.first_name || 'User'
})

// Watch for edit mode
watch(() => store.editingMessage, (newMsg) => {
  if (newMsg) {
    text.value = newMsg.body
    nextTick(() => inputRef.value?.focus())
  } else {
    text.value = ''
  }
})

// Focus when replying
watch(() => store.replyingToMessage, (newMsg) => {
  if (newMsg) {
    nextTick(() => inputRef.value?.focus())
  }
})

function handleInput() {
  if (!isTypingActive) {
    isTypingActive = true
    emit('typing')
  }
  if (typingTimer) clearTimeout(typingTimer)
  typingTimer = setTimeout(() => {
    isTypingActive = false
    emit('stopTyping')
  }, 2000)
}

function submit() {
  const t = text.value.trim()
  if (!t) return

  if (store.editingMessage) {
    store.editMessage(store.editingMessage.uuid, t)
    store.editingMessage = null
  } else {
    emit('send', t)
  }
  
  text.value = ''
  isTypingActive = false
  emit('stopTyping')
  if (typingTimer) clearTimeout(typingTimer)
  nextTick(() => inputRef.value?.focus())
}

function closeEdit() {
  store.editingMessage = null
  text.value = ''
}

function insertEmoji(emoji: string) {
  text.value += emoji
  showEmoji.value = false
  nextTick(() => inputRef.value?.focus())
}

function handleVoiceSend(blob: Blob) {
  store.sendVoiceMessage(blob)
}

function triggerFileSelect() {
  fileInput.value?.click()
}

function handleFileChange(e: Event) {
  const files = (e.target as HTMLInputElement).files
  if (files && files[0]) {
    store.sendFileMessage(files[0])
    if (fileInput.value) fileInput.value.value = ''
  }
}

function openPicker(mode: 'gif' | 'sticker') {
  if (showGif.value && pickerMode.value === mode) {
    showGif.value = false
  } else {
    pickerMode.value = mode
    showGif.value = true
  }
}

function handleGifSelect(md: string) {
  emit('send', md)
  showGif.value = false
}

defineExpose({
  focus: () => {
    nextTick(() => inputRef.value?.focus())
  }
})
</script>

<template>
  <div class="flex flex-col border-t border-white/5 glass-heavy relative z-20 backdrop-blur-3xl">
    
    <!-- Reply Preview Area -->
    <div 
      v-if="store.replyingToMessage" 
      class="px-5 py-2.5 flex items-center justify-between bg-white/5 border-b border-white/5 animate-in slide-in-from-bottom-2 duration-200"
    >
      <div class="flex items-center gap-3 overflow-hidden">
        <div class="w-1 h-8 bg-blue-500 rounded-full"></div>
        <div class="flex flex-col min-w-0">
          <span class="text-[10px] font-mono font-black uppercase tracking-tight text-blue-400">Replying to {{ replySender }}</span>
          <span class="text-[11px] font-mono text-text-dim truncate">{{ store.replyingToMessage.body }}</span>
        </div>
      </div>
      <button @click="store.replyingToMessage = null" class="p-1 hover:bg-white/10 rounded-lg text-text-dim">
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>

    <!-- Edit Mode Indicator -->
    <div 
      v-if="store.editingMessage" 
      class="px-5 py-2.5 flex items-center justify-between bg-pulse/10 border-b border-pulse/20 animate-in slide-in-from-bottom-2 duration-200"
    >
      <div class="flex items-center gap-3">
        <svg class="w-4 h-4 text-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
        </svg>
        <span class="text-[10px] font-mono font-black uppercase tracking-wider text-pulse">Editing Message</span>
      </div>
      <button @click="closeEdit" class="p-1 hover:bg-white/10 rounded-lg text-pulse">
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>

    <div class="flex items-end gap-3 px-5 py-5 z-20">
      <!-- File button -->
      <button
        @click="triggerFileSelect"
        class="shrink-0 p-2.5 rounded-xl text-text-dim hover:text-blue-400 hover:bg-blue-400/10 border border-transparent hover:border-blue-400/20 transition-all group"
      >
        <svg class="w-5 h-5 group-hover:rotate-12 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
        </svg>
        <input ref="fileInput" type="file" class="hidden" @change="handleFileChange" />
      </button>

      <!-- Emoji button -->
      <button
        @click="showEmoji = !showEmoji"
        class="shrink-0 p-2.5 rounded-xl text-text-dim hover:text-pulse hover:bg-pulse/10 border border-transparent hover:border-pulse/20 transition-all group"
      >
        <svg class="w-5 h-5 group-hover:scale-110 transition-transform" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM7 9a1 1 0 100-2 1 1 0 000 2zm7-1a1 1 0 11-2 0 1 1 0 012 0zm-.464 5.535a1 1 0 10-1.415-1.414 3 3 0 01-4.242 0 1 1 0 00-1.415 1.414 5 5 0 007.072 0z" clip-rule="evenodd"/>
        </svg>
      </button>

      <!-- GIF button -->
      <button
        @click="openPicker('gif')"
        class="shrink-0 p-2.5 rounded-xl text-text-dim hover:text-ember hover:bg-ember/10 border border-transparent hover:border-ember/20 transition-all group"
        title="GIFs"
      >
        <span class="text-[10px] font-black group-hover:scale-110 transition-transform">GIF</span>
      </button>

      <!-- Sticker button -->
      <button
        @click="openPicker('sticker')"
        class="shrink-0 p-2.5 rounded-xl text-text-dim hover:text-ember hover:bg-ember/10 border border-transparent hover:border-ember/20 transition-all group"
        title="Stickers"
      >
        <span class="text-lg leading-none group-hover:scale-110 transition-transform">✨</span>
      </button>

      <!-- GIF/Sticker picker -->
      <div v-if="showGif" class="absolute bottom-24 left-6 z-30">
        <GifPicker :initial-mode="pickerMode" @select="handleGifSelect" @close="showGif = false" />
      </div>

      <!-- Emoji picker -->
      <Transition name="emoji-pop">
        <div
          v-if="showEmoji"
          class="absolute bottom-24 left-6 z-30 grid grid-cols-4 sm:grid-cols-8 gap-1 p-4 glass-heavy border border-white/10 rounded-2xl shadow-2xl backdrop-blur-3xl"
        >
          <button
            v-for="e in EMOJIS"
            :key="e"
            @click="insertEmoji(e)"
            class="text-lg hover:scale-125 hover:bg-white/5 transition-all p-1.5 rounded-lg"
          >{{ e }}</button>
        </div>
      </Transition>

      <!-- Text area -->
      <div class="flex-1 relative">
        <textarea
          ref="inputRef"
          v-model="text"
          @keydown.enter.exact.prevent="submit"
          @keydown.shift.enter="() => {}"
          @input="handleInput"
          @contextmenu="handleInputContextMenu"
          rows="1"
          :placeholder="store.editingMessage ? 'Update message...' : 'Broadcast message...'"
          class="w-full bg-white/3 border border-white/5 rounded-2xl px-5 py-3.5 text-sm text-text-bright placeholder:text-muted outline-none focus:border-pulse/40 focus:bg-white/5 resize-none transition-all font-mono leading-relaxed overflow-hidden shadow-inner"
          :style="{ height: textareaHeight }"
        />
      </div>

      <!-- Voice Recorder -->
      <VoiceRecorder 
        v-if="!text.trim() && !store.editingMessage"
        @send="handleVoiceSend"
        class="shrink-0"
      />

      <!-- Send button -->
      <button
        v-else
        @click="submit"
        :disabled="!text.trim()"
        class="shrink-0 p-3.5 rounded-xl transition-all duration-300 relative group"
        :class="text.trim()
          ? 'bg-pulse text-white shadow-[0_0_20px_rgba(var(--color-pulse),0.4)] hover:shadow-[0_0_25px_rgba(var(--color-pulse),0.6)] hover:scale-105'
          : 'bg-white/5 text-muted border border-white/5 cursor-not-allowed'"
      >
        <svg class="w-5 h-5 group-hover:translate-x-0.5 transition-transform" viewBox="0 0 20 20" fill="currentColor">
          <path v-if="store.editingMessage" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" />
          <path v-else d="M3.105 2.289a.75.75 0 00-.826.95l1.414 4.925A1.5 1.5 0 005.135 9.25h6.115a.75.75 0 010 1.5H5.135a1.5 1.5 0 00-1.442 1.086l-1.414 4.926a.75.75 0 00.826.95 28.896 28.896 0 0015.293-7.154.75.75 0 000-1.115A28.897 28.897 0 003.105 2.289z"/>
        </svg>
      </button>
    </div>

    <Teleport to="body">
      <ContextMenu
        v-if="showFormattingMenu"
        :items="formattingItems"
        :x="menuX"
        :y="menuY"
        @close="showFormattingMenu = false"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.emoji-pop-enter-active, .emoji-pop-leave-active {
  transition: opacity 0.15s, transform 0.1s cubic-bezier(0.2, 0, 0, 1);
}
.emoji-pop-enter-from, .emoji-pop-leave-to {
  opacity: 0;
  transform: translateY(12px) scale(0.9);
}
</style>
