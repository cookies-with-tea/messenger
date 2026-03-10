<template>
  <div class="flex items-end gap-3 px-4 py-4 border-t border-border bg-abyss">
    <!-- Emoji button -->
    <button
      @click="showEmoji = !showEmoji"
      class="shrink-0 p-2.5 rounded-xl text-text-dim hover:text-text-bright hover:bg-elevated transition-all"
    >
      <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM7 9a1 1 0 100-2 1 1 0 000 2zm7-1a1 1 0 11-2 0 1 1 0 012 0zm-.464 5.535a1 1 0 10-1.415-1.414 3 3 0 01-4.242 0 1 1 0 00-1.415 1.414 5 5 0 007.072 0z" clip-rule="evenodd"/>
      </svg>
    </button>

    <!-- Emoji picker -->
    <Transition name="emoji-pop">
      <div
        v-if="showEmoji"
        class="absolute bottom-20 left-16 z-20 grid grid-cols-8 gap-1 p-3 bg-elevated border border-border rounded-2xl shadow-xl"
      >
        <button
          v-for="e in EMOJIS"
          :key="e"
          @click="insertEmoji(e)"
          class="text-lg hover:scale-125 transition-transform p-0.5"
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
        rows="1"
        placeholder="Message..."
        class="w-full bg-surface border border-border rounded-2xl px-4 py-3 text-sm text-text-bright placeholder:text-muted outline-none focus:border-pulse/50 focus:bg-elevated resize-none transition-all font-mono leading-relaxed overflow-hidden"
        :style="{ height: textareaHeight }"
      />
    </div>

    <!-- Voice Recorder -->
    <VoiceRecorder 
      v-if="!text.trim()"
      @send="handleVoiceSend"
      class="shrink-0"
    />

    <!-- Send button -->
    <button
      v-else
      @click="submit"
      :disabled="!text.trim()"
      class="shrink-0 p-2.5 rounded-xl transition-all duration-200"
      :class="text.trim()
        ? 'bg-pulse text-white hover:bg-pulse-glow shadow-glow'
        : 'bg-surface text-muted border border-border cursor-not-allowed'"
    >
      <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
        <path d="M3.105 2.289a.75.75 0 00-.826.95l1.414 4.925A1.5 1.5 0 005.135 9.25h6.115a.75.75 0 010 1.5H5.135a1.5 1.5 0 00-1.442 1.086l-1.414 4.926a.75.75 0 00.826.95 28.896 28.896 0 0015.293-7.154.75.75 0 000-1.115A28.897 28.897 0 003.105 2.289z"/>
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { useMessengerStore } from '@/stores/messengerStore'
import VoiceRecorder from './VoiceRecorder.vue'

const messenger = useMessengerStore()

const EMOJIS = ['😀','😂','🥹','😎','🤔','🚀','💡','🔥','❤️','👍','👎','🎉','⚡','🌊','🎮','🎨','💻','🤖','👾','⭐','✨','🎯','📱','🏆']

const emit = defineEmits<{
  send: [text: string]
  typing: []
  stopTyping: []
}>()

const text = ref('')
const showEmoji = ref(false)
const inputRef = ref<HTMLTextAreaElement>()
let typingTimer: ReturnType<typeof setTimeout> | null = null
let isTypingActive = false

const textareaHeight = computed(() => {
  const lines = (text.value.match(/\n/g) ?? []).length + 1
  return `${Math.min(lines * 24 + 24, 120)}px`
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
  emit('send', t)
  text.value = ''
  isTypingActive = false
  emit('stopTyping')
  if (typingTimer) clearTimeout(typingTimer)
  nextTick(() => inputRef.value?.focus())
}

function insertEmoji(emoji: string) {
  text.value += emoji
  showEmoji.value = false
  nextTick(() => inputRef.value?.focus())
}

function handleVoiceSend(blob: Blob) {
  messenger.sendVoiceMessage(blob)
}
</script>

<style scoped>
.emoji-pop-enter-active, .emoji-pop-leave-active {
  transition: opacity 0.15s, transform 0.15s;
}
.emoji-pop-enter-from, .emoji-pop-leave-to {
  opacity: 0;
  transform: translateY(8px) scale(0.95);
}
</style>
