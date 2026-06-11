<script setup>
import { ref, reactive, watch } from 'vue'
import Modal from '../common/Modal.vue'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()

const props = defineProps({
  visible: { type: Boolean, default: false },
  connection: { type: Object, default: null },
  showDelete: { type: Boolean, default: false },
})

const emit = defineEmits(['close', 'saved', 'delete'])

const form = reactive({
  name: '',
  host: '',
  port: 22,
  username: '',
  authMethod: 'password',
  password: '',
  privateKeyPath: '',
  passphrase: '',
  keepaliveSeconds: null,
})

const errors = ref({})

watch(() => props.connection, (conn) => {
  if (conn) {
    Object.assign(form, {
      name: conn.name || '',
      host: conn.host || '',
      port: conn.port || 22,
      username: conn.username || '',
      authMethod: conn.auth_method?.type || 'password',
      password: conn.auth_method?.type === 'password' ? (conn.auth_method.value?.password || '') : '',
      privateKeyPath: conn.auth_method?.type === 'private_key' ? (conn.auth_method.value?.private_key_path || '') : '',
      passphrase: conn.auth_method?.type === 'private_key' ? (conn.auth_method.value?.passphrase || '') : '',
      keepaliveSeconds: conn.keepalive_seconds || null,
    })
    errors.value = {}
  } else {
    Object.assign(form, {
      name: '', host: '', port: 22, username: '',
      authMethod: 'password', password: '', privateKeyPath: '',
      passphrase: '', keepaliveSeconds: null,
    })
  }
}, { immediate: true })

function validate() {
  const e = {}
  if (!form.name.trim()) e.name = t('connection.nameRequired')
  if (!form.host.trim()) e.host = t('connection.hostRequired')
  if (!form.port || form.port < 1 || form.port > 65535) e.port = t('connection.portInvalid')
  if (!form.username.trim()) e.username = t('connection.userRequired')
  if (form.authMethod === 'password' && !form.password) e.password = t('connection.pwRequired')
  if (form.authMethod === 'private_key' && !form.privateKeyPath.trim()) e.privateKeyPath = t('connection.keyRequired')
  errors.value = e
  return Object.keys(e).length === 0
}

function onSubmit() {
  if (!validate()) return

  const id = props.connection?.id || ''
  const authMethod = form.authMethod === 'password'
    ? { type: 'password', value: { password: form.password } }
    : { type: 'private_key', value: { private_key_path: form.privateKeyPath, passphrase: form.passphrase || null } }

  emit('saved', {
    id,
    name: form.name.trim(),
    host: form.host.trim(),
    port: Number(form.port),
    username: form.username.trim(),
    auth_method: authMethod,
    group: null,
    keepalive_seconds: form.keepaliveSeconds || null,
    last_connected: props.connection?.last_connected || null,
  })
}
</script>

<template>
  <Modal :visible="visible" :title="t(connection?.id ? 'connection.editTitle' : 'connection.newTitle')" @close="emit('close')">
    <form @submit.prevent="onSubmit" class="space-y-3">
      <div>
        <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.name') }}</label>
        <input v-model="form.name" type="text" placeholder="My Server"
          class="w-full px-3 py-1.5 text-sm rounded-[var(--radius-sm)] border border-[var(--color-border)]
            bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
            focus:outline-none focus:border-[var(--color-accent)] transition-colors" />
        <p v-if="errors.name" class="text-xs text-[var(--color-danger)] mt-0.5">{{ errors.name }}</p>
      </div>

      <div class="flex gap-3">
        <div class="flex-1">
          <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.host') }}</label>
          <input v-model="form.host" type="text" placeholder="192.168.1.5"
            class="w-full px-3 py-1.5 text-sm rounded-[var(--radius-sm)] border border-[var(--color-border)]
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
              focus:outline-none focus:border-[var(--color-accent)] transition-colors" />
          <p v-if="errors.host" class="text-xs text-[var(--color-danger)] mt-0.5">{{ errors.host }}</p>
        </div>
        <div class="w-20">
          <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.port') }}</label>
          <input v-model.number="form.port" type="number"
            class="w-full px-3 py-1.5 text-sm rounded-[var(--radius-sm)] border border-[var(--color-border)]
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
              focus:outline-none focus:border-[var(--color-accent)] transition-colors" />
          <p v-if="errors.port" class="text-xs text-[var(--color-danger)] mt-0.5">{{ errors.port }}</p>
        </div>
      </div>

      <div>
        <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.username') }}</label>
        <input v-model="form.username" type="text" placeholder="root"
          class="w-full px-3 py-1.5 text-sm rounded-[var(--radius-sm)] border border-[var(--color-border)]
            bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
            focus:outline-none focus:border-[var(--color-accent)] transition-colors" />
        <p v-if="errors.username" class="text-xs text-[var(--color-danger)] mt-0.5">{{ errors.username }}</p>
      </div>

      <div>
        <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{{ t('connection.auth') }}</label>
        <div class="flex gap-1 bg-[var(--color-bg-secondary)] rounded-[var(--radius-sm)] p-0.5">
          <button type="button" @click="form.authMethod = 'password'"
            :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
              form.authMethod === 'password' ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]' : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
          >{{ t('connection.password') }}</button>
          <button type="button" @click="form.authMethod = 'private_key'"
            :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
              form.authMethod === 'private_key' ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]' : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
          >{{ t('connection.privateKey') }}</button>
        </div>
      </div>

      <template v-if="form.authMethod === 'password'">
        <div>
          <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.password') }}</label>
          <input v-model="form.password" type="password" placeholder="Enter password"
            class="w-full px-3 py-1.5 text-sm rounded-[var(--radius-sm)] border border-[var(--color-border)]
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
              focus:outline-none focus:border-[var(--color-accent)] transition-colors" />
          <p v-if="errors.password" class="text-xs text-[var(--color-danger)] mt-0.5">{{ errors.password }}</p>
        </div>
      </template>

      <template v-if="form.authMethod === 'private_key'">
        <div>
          <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.keyPath') }}</label>
          <input v-model="form.privateKeyPath" type="text" placeholder="~/.ssh/id_rsa"
            class="w-full px-3 py-1.5 text-sm rounded-[var(--radius-sm)] border border-[var(--color-border)]
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
              focus:outline-none focus:border-[var(--color-accent)] transition-colors" />
          <p v-if="errors.privateKeyPath" class="text-xs text-[var(--color-danger)] mt-0.5">{{ errors.privateKeyPath }}</p>
        </div>
        <div>
          <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.passphrase') }}</label>
          <input v-model="form.passphrase" type="password" placeholder="Key passphrase"
            class="w-full px-3 py-1.5 text-sm rounded-[var(--radius-sm)] border border-[var(--color-border)]
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
              focus:outline-none focus:border-[var(--color-accent)] transition-colors" />
        </div>
      </template>

      <!-- Keepalive -->
      <div>
        <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1">{{ t('connection.keepalive') }}</label>
        <div class="flex items-center gap-1.5">
          <input v-model.number="form.keepaliveSeconds" type="number" min="0" max="3600" placeholder="0"
            class="w-16 px-2 py-1 text-xs text-right rounded-lg border border-[var(--color-border)]
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] font-mono
              focus:outline-none focus:border-[var(--color-accent)] transition-all" />
          <span class="text-[10px] text-[var(--color-text-tertiary)]">s (0 = {{ t('connection.keepaliveOff') }})</span>
        </div>
      </div>

      <div class="flex justify-between gap-2 pt-2">
        <button v-if="showDelete && connection?.id" type="button"
          @click="emit('delete', connection.id)"
          class="px-4 py-1.5 text-xs font-medium rounded-[var(--radius-sm)]
            text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors"
        >{{ t('connection.delete') }}</button>
        <span v-else />
        <div class="flex gap-2">
          <button type="button" @click="emit('close')"
            class="px-4 py-1.5 text-xs font-medium rounded-[var(--radius-sm)]
              text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)] transition-colors"
          >{{ t('connection.cancel') }}</button>
          <button type="submit"
            class="px-4 py-1.5 text-xs font-medium rounded-[var(--radius-sm)]
              bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)] transition-colors"
          >{{ t(connection?.id ? 'connection.save' : 'connection.add') }}</button>
        </div>
      </div>
    </form>
  </Modal>
</template>
