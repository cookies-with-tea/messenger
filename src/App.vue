<script setup lang="ts">
import ToastContainer from '@/components/ui/ToastContainer.vue';
import GlobalSearchModal from '@/components/GlobalSearchModal.vue';
import { useMessengerStore } from '@/stores/messengerStore';

const messenger = useMessengerStore();
</script>

<template>
  <RouterView />
  <ToastContainer />
  <GlobalSearchModal 
    :is-open="messenger.isGlobalSearchModalOpen" 
    @close="messenger.isGlobalSearchModalOpen = false" 
  />
</template>
