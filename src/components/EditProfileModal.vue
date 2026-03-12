<template>
  <Transition name="modal">
    <div v-if="isOpen" class="modal-overlay" @click.self="close">
      <div class="modal-content glass">
        <div class="modal-header">
          <h3>Edit Profile</h3>
          <button class="close-btn" @click="close">
            <i class="fas fa-times"></i>
          </button>
        </div>

        <form @submit.prevent="handleSave" class="edit-form">
          <div class="avatar-section">
            <div class="avatar-preview glass">
              <img v-if="form.avatar" :src="form.avatar" alt="Avatar" />
              <div v-else class="avatar-placeholder">
                <i class="fas fa-user"></i>
              </div>
              <label class="avatar-upload-btn">
                <i class="fas fa-camera"></i>
                <input type="file" @change="handleAvatarUpload" accept="image/*" hidden />
              </label>
            </div>
            <p class="avatar-hint">Click the camera to change photo</p>
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label>First Name</label>
              <input v-model="form.first_name" type="text" placeholder="First Name" class="glass-input" />
            </div>
            <div class="form-group">
              <label>Last Name</label>
              <input v-model="form.last_name" type="text" placeholder="Last Name" class="glass-input" />
            </div>
            <div class="form-group full-width">
              <label>Email (Cannot be changed)</label>
              <input :value="user?.email" type="email" disabled class="glass-input disabled" />
            </div>
            <div class="form-group">
              <label>Phone</label>
              <input v-model="form.phone" type="tel" placeholder="Phone" class="glass-input" />
            </div>
            <div class="form-group">
              <label>City</label>
              <input v-model="form.city" type="text" placeholder="City" class="glass-input" />
            </div>
            <div class="form-group full-width">
              <label>Street</label>
              <input v-model="form.street" type="text" placeholder="Street" class="glass-input" />
            </div>
          </div>

          <div class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="close">Cancel</button>
            <button type="submit" class="btn btn-primary" :disabled="loading">
              <span v-if="loading">Saving...</span>
              <span v-else>Save Changes</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { useMessengerStore } from '@/stores/messengerStore';
import { mediaApi } from '@/api';

const props = defineProps<{
  isOpen: boolean;
  user: any;
}>();

const emit = defineEmits(['close']);
const store = useMessengerStore();
const loading = ref(false);

const form = reactive({
  first_name: '',
  last_name: '',
  phone: '',
  city: '',
  street: '',
  avatar: '',
});

watch(() => props.isOpen, (val) => {
  if (val && props.user) {
    form.first_name = props.user.first_name || '';
    form.last_name = props.user.last_name || '';
    form.phone = props.user.phone || '';
    form.city = props.user.city || '';
    form.street = props.user.street || '';
    form.avatar = props.user.avatar || '';
  }
}, { immediate: true });

function close() {
  emit('close');
}

async function handleAvatarUpload(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;

  try {
    loading.value = true;
    const res = await mediaApi.upload(file);
    if (res.data?.url) {
      form.avatar = res.data.url;
    }
  } catch (e) {
    console.error('Failed to upload avatar:', e);
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  loading.value = true;
  try {
    await store.updateProfile({ ...form });
    close();
  } catch (e) {
    console.error('Failed to save profile:', e);
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-content {
  width: 100%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
  border-radius: 24px;
  padding: 24px;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  position: sticky;
  top: 0;
  z-index: 10;
  padding: 24px;
  margin: -24px -24px 24px -24px;
  background: rgba(255, 255, 255, 0.02);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.modal-header h3 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.close-btn {
  background: transparent;
  border: none;
  color: white;
  cursor: pointer;
  font-size: 1.2rem;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.close-btn:hover {
  opacity: 1;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 32px;
}

.avatar-preview {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 12px;
  border: 4px solid rgba(255, 255, 255, 0.1);
}

.avatar-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder {
  font-size: 3rem;
  opacity: 0.5;
}

.avatar-upload-btn {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.3s;
}

.avatar-preview:hover .avatar-upload-btn {
  opacity: 1;
}

.avatar-hint {
  font-size: 0.8rem;
  opacity: 0.7;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 32px;
}

.full-width {
  grid-column: span 2;
}

.form-group label {
  display: block;
  font-size: 0.9rem;
  margin-bottom: 8px;
  opacity: 0.8;
}

.glass-input {
  width: 100%;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  color: white;
  font-size: 1rem;
  transition: all 0.3s;
}

.glass-input:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.1);
  border-color: var(--primary-color, #6c5ce7);
  box-shadow: 0 0 0 4px rgba(108, 92, 231, 0.1);
}

.glass-input.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  position: sticky;
  bottom: 0;
  z-index: 10;
  padding: 24px;
  margin: 24px -24px -24px -24px;
  background: rgba(255, 255, 255, 0.02);
  backdrop-filter: blur(20px);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.btn {
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
  border: none;
}

.btn-primary {
  background: var(--primary-color, #6c5ce7);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #5b4cc4;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(108, 92, 231, 0.3);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.2);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Modal animation */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
</style>
