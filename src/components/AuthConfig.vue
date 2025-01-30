<template>
    <div class="auth-config">
        <div class="title">
            <n-icon size="18">
                <key-outlined />
            </n-icon>
            <span>{{ t('authConfig.title') }}</span>
        </div>
        <div class="inputs">
            <n-input v-model:value="email" type="text" :placeholder="t('authConfig.emailPlaceholder')" size="small"
                style="width: 200px" />
            <n-input v-model:value="token" type="text" :placeholder="t('authConfig.tokenPlaceholder')" size="small"
                style="width: 200px" />
            <n-button type="primary" @click="handleUpdate" :loading="isUpdating" :disabled="!canUpdate" size="small">
                {{ t('common.apply') }}
            </n-button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { NInput, NButton, NIcon, useMessage } from 'naive-ui'
import { KeyOutlined } from '@vicons/antd'

defineOptions({
    name: 'AuthConfig'
})

const message = useMessage()
const { t } = useI18n()

const email = ref('')
const token = ref('')
const isUpdating = ref(false)

const canUpdate = computed(() => email.value.trim() !== '' && token.value.trim() !== '')

const handleUpdate = async () => {
    if (!canUpdate.value) return

    isUpdating.value = true
    try {
        await invoke('update_auth', {
            email: email.value.trim(),
            token: token.value.trim()
        })
        message.success(t('authConfig.updateSuccess'))
        email.value = ''
        token.value = ''
    } catch (error) {
        message.error(t('authConfig.updateFailed') + ': ' + (error instanceof Error ? error.message : t('common.error')))
    } finally {
        isUpdating.value = false
    }
}
</script>

<style scoped>
.auth-config {
    display: flex;
    flex-direction: column;
    max-width: 800px;
    margin: 0 auto;
    gap: 8px;
    padding-top: 12px;
}

.title {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-color);
    font-weight: 500;
    font-size: 14px;
}

.inputs {
    display: flex;
    align-items: center;
    gap: 12px;
}
</style>