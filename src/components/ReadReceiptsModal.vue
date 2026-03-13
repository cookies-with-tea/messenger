<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { messageApi } from '@/api';
import type { MessageReceiptDTO } from '@/types';

const props = defineProps<{
  chatUuid: string;
  messageUuid: string;
  isOpen: boolean;
}>();

const emit = defineEmits(['close']);

const receipts = ref<MessageReceiptDTO[]>([]);
const isLoading = ref(true);

async function fetchReceipts() {
  if (!props.chatUuid || !props.messageUuid) return;
  isLoading.value = true;
  try {
    const res = await messageApi.getMessageReceipts(props.chatUuid, props.messageUuid);
    if (res.data) {
      receipts.value = res.data;
    }
  } catch (err) {
    console.error('Failed to fetch receipts:', err);
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => {
  fetchReceipts();
});

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString();
}
</script>

<template>
  <div v-if="props.isOpen" class="modal-overlay" @click="$emit('close')">
    <div class="modal-content glass" @click.stop>
      <div class="modal-header">
        <h3>Message Status</h3>
        <button class="close-btn" @click="$emit('close')">&times;</button>
      </div>

      <div class="modal-body custom-scrollbar">
        <div v-if="isLoading" class="loading">
          <div class="spinner"></div>
          <span>Loading statuses...</span>
        </div>

        <div v-else-if="receipts.length === 0" class="empty">
          No status information available.
        </div>

        <div v-else class="receipts-list">
          <div v-for="receipt in receipts" :key="receipt.user.uuid" class="receipt-item">
            <div class="user-avatar">
              <img v-if="receipt.user.avatar" :src="receipt.user.avatar.url" :alt="receipt.user.first_name || 'User'" />
              <div v-else class="avatar-placeholder">
                {{ (receipt.user.first_name?.[0] || '?').toUpperCase() }}
              </div>
              <div class="online-indicator" :class="{ online: receipt.user.is_online }"></div>
            </div>
            
            <div class="receipt-info">
              <div class="user-name">
                {{ receipt.user.first_name }} {{ receipt.user.second_name }}
              </div>
              <div class="status-time">
                <span class="status-badge" :class="receipt.status">
                  {{ receipt.status }}
                </span>
                <span class="time">{{ formatDate(receipt.created_at) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  backdrop-filter: blur(4px);
}

.modal-content {
  width: 90%;
  max-width: 400px;
  max-height: 80vh;
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.modal-header {
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.modal-header h3 {
  margin: 0;
  font-size: 1.2rem;
  color: #fff;
}

.close-btn {
  background: none;
  border: none;
  color: #aaa;
  font-size: 24px;
  cursor: pointer;
  padding: 4px;
}

.modal-body {
  padding: 16px;
  overflow-y: auto;
  flex: 1;
}

.receipts-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.receipt-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
}

.user-avatar {
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: 50%;
}

.user-avatar img,
.avatar-placeholder {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
}

.avatar-placeholder {
  background: #6366f1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  color: white;
}

.online-indicator {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #666;
  border: 2px solid #1a1a1a;
}

.online-indicator.online {
  background: #4caf50;
}

.receipt-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.user-name {
  font-weight: 500;
  color: #eee;
}

.status-time {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
}

.status-badge {
  text-transform: capitalize;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: bold;
}

.status-badge.read {
  color: #4fc3f7;
  background: rgba(79, 195, 247, 0.1);
}

.status-badge.delivered {
  color: #aaa;
  background: rgba(255, 255, 255, 0.05);
}

.time {
  color: #888;
}

.loading, .empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: #888;
  gap: 12px;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-top-color: #6366f1;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.glass {
  background: rgba(30, 30, 35, 0.8);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
