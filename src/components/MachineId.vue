<template>
    <n-card class="machine-id" :bordered="false" size="small">
        <div class="content-wrapper">
            <div class="progress-section">
                <div class="section-title">
                    <n-icon size="14"><info-outlined /></n-icon>
                    {{ t('machineId.progress') }}
                </div>
                <n-scrollbar ref="scrollbarRef" class="message-container" trigger="none">
                    <div class="message-wrapper">
                        <n-text v-for="(msg, index) in progressMessages" :key="index"
                            :depth="index === progressMessages.length - 1 ? 1 : 3"
                            :type="index === progressMessages.length - 1 ? 'primary' : undefined" class="message-item">
                            {{ msg }}
                        </n-text>
                    </div>
                </n-scrollbar>
            </div>

            <div class="actions">
                <div class="action-buttons">
                    <n-button type="primary" @click="resetId" :loading="resetLoading" class="action-btn" size="small">
                        <template #icon><reload-outlined /></template>
                        {{ t('machineId.resetId') }}
                    </n-button>
                    <n-button @click="modifyId" :loading="modifyLoading" :disabled="modifyLoading" class="action-btn"
                        size="small">
                        <template #icon><edit-outlined /></template>
                        {{ t('machineId.modifyId') }}
                    </n-button>
                    <n-button @click="backup" :loading="backupLoading" :disabled="backupLoading" class="action-btn"
                        size="small">
                        <template #icon><save-outlined /></template>
                        {{ t('machineId.backup') }}
                    </n-button>
                    <n-button @click="restore" :loading="restoreLoading" :disabled="restoreLoading" class="action-btn"
                        size="small">
                        <template #icon><rollback-outlined /></template>
                        {{ t('machineId.restore') }}
                    </n-button>
                </div>
            </div>
            <div class="config-section">
                <div class="section-title">
                    <n-icon size="14"><folder-outlined /></n-icon>
                    {{ t('machineId.configPath') }}
                </div>
                <n-input readonly :value="configPath" :placeholder="t('machineId.configPath')" :border="false"
                    class="config-input" size="small" />
                <div class="id-grid">
                    <div class="id-item" v-for="(id, key) in configIds" :key="key">
                        <div class="id-label">{{ labels[key] }}</div>
                        <n-input readonly :value="id" :placeholder="labels[key]" :border="false" class="id-input"
                            size="small" />
                    </div>
                </div>
            </div>

            <div class="config-section">
                <div class="section-title">
                    <n-icon size="14"><folder-outlined /></n-icon>
                    {{ t('machineId.mainJsPath') }}
                </div>
                <n-input readonly :value="mainJsPath" :placeholder="t('machineId.mainJsPath')" :border="false"
                    class="config-input" size="small" />
                <div class="id-grid">
                    <div class="id-item" v-for="(id, key) in mainJsIds" :key="key">
                        <div class="id-label">{{ labels[key] }}</div>
                        <n-input readonly :value="id" :placeholder="labels[key]" :border="false" class="id-input"
                            size="small" />
                    </div>
                </div>
            </div>

        </div>
    </n-card>
</template>

<script setup lang="ts">

import { ref, onMounted, computed, type Ref, onUnmounted, nextTick } from '@vue/runtime-core'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import {
    NButton,
    NInput,
    NCard,
    NIcon,
    NText,
    NScrollbar,
    useMessage,
    useThemeVars
} from 'naive-ui'
import {
    FolderOutlined,
    KeyOutlined,
    ReloadOutlined,
    EditOutlined,
    SaveOutlined,
    RollbackOutlined,
    InfoOutlined
} from '@vicons/antd'

defineOptions({
    name: 'MachineId'
})

interface MachineIds {
    machineId: string
    macMachineId: string
    devDeviceId: string
    sqmId: string
    configPath: string
}

interface IdsMap {
    machineId: string
    macMachineId: string
    devDeviceId: string
    sqmId: string
}

interface MainJsIds {
    machineId: string
    macMachineId: string
    devDeviceId: string
    sqmId: string
    jsPath: string
}

const message = useMessage()
const themeVars = useThemeVars()
const { t } = useI18n()

// loading 状态
const resetLoading = ref<boolean>(false)
const modifyLoading = ref<boolean>(false)
const backupLoading = ref<boolean>(false)
const restoreLoading = ref<boolean>(false)

// 进度信息
const progressMessage = ref<string>('')
const progressMessages = ref<string[]>([])

// ID 相关状态
const configPath = ref<string>('')
const mainJsPath = ref<string>('')
const machineId = ref<string>('')
const macMachineId = ref<string>('')
const devDeviceId = ref<string>('')
const sqmId = ref<string>('')
const mainJsMachineId = ref<string>('')
const mainJsMacMachineId = ref<string>('')
const mainJsDevDeviceId = ref<string>('')
const mainJsSqmId = ref<string>('')

const configIds = computed<IdsMap>(() => ({
    machineId: machineId.value,
    macMachineId: macMachineId.value,
    devDeviceId: devDeviceId.value,
    sqmId: sqmId.value
}))

const mainJsIds = computed<IdsMap>(() => ({
    machineId: mainJsMachineId.value,
    macMachineId: mainJsMacMachineId.value,
    devDeviceId: mainJsDevDeviceId.value,
    sqmId: mainJsSqmId.value
}))

const labels: Record<keyof IdsMap, string> = {
    machineId: 'Machine ID',
    macMachineId: 'Mac Machine ID',
    devDeviceId: 'Dev Device ID',
    sqmId: 'SQM ID'
}

// 获取 main.js 中的 ID
const getMainJsIds = async () => {
    try {
        const result = await invoke<MainJsIds>('get_mainjs_ids')
        mainJsMachineId.value = result.machineId
        mainJsMacMachineId.value = result.macMachineId
        mainJsDevDeviceId.value = result.devDeviceId
        mainJsSqmId.value = result.sqmId
        mainJsPath.value = result.jsPath
    } catch (e) {
        console.error('获取main.js中的ID失败:', e)
        message.error(t('machineId.getMainJsIdsFailed') + ': ' + e)
    }
}

// 更新所有 ID 信息
const getIds = async () => {
    try {
        const result = await invoke<MachineIds>('get_all_ids')
        machineId.value = result.machineId
        macMachineId.value = result.macMachineId
        devDeviceId.value = result.devDeviceId
        sqmId.value = result.sqmId
        configPath.value = result.configPath
    } catch (e) {
        console.error('获取ID信息失败:', e)
        message.error(t('machineId.getIdsFailed') + ': ' + e)
    }
}

// 执行操作的通用方法
const executeAction = async (
    action: () => Promise<void>,
    loadingRef: Ref<boolean>,
    successMessage: string,
    errorMessage: string,
    shouldRefreshIds = true
) => {
    loadingRef.value = true
    try {
        await action()
        if (shouldRefreshIds) {
            await getIds()
        }
        message.success(t(successMessage))
    } catch (e) {
        console.error(errorMessage, e)
        message.error(t(errorMessage) + ': ' + e)
    } finally {
        loadingRef.value = false
    }
}

// 重置 ID
const resetId = () => executeAction(
    () => invoke('reset_machine_id'),
    resetLoading,
    'machineId.resetSuccess',
    'machineId.resetFailed'
)

// 修改 ID
const modifyId = () => executeAction(
    async () => {
        await invoke('update_machine_id')
        await getMainJsIds() // 更新 main.js 中的 ID
    },
    modifyLoading,
    'machineId.modifySuccess',
    'machineId.modifyFailed'
)

// 备份配置
const backup = () => executeAction(
    async () => {
        await invoke('backup_config', { appHandle: {} })  // 传递空对象作为app_handle
        await getIds()  // 刷新配置文件的ID
        await getMainJsIds()  // 刷新main.js的ID
    },
    backupLoading,
    'machineId.backupSuccess',
    'machineId.backupFailed',
    false
)

// 还原配置
const restore = () => executeAction(
    async () => {
        await invoke('restore_config', { appHandle: {} })  // 传递空对象作为app_handle
        await getIds()  // 刷新配置文件的ID
        await getMainJsIds()  // 刷新main.js的ID
    },
    restoreLoading,
    'machineId.restoreSuccess',
    'machineId.restoreFailed'
)

const scrollbarRef = ref<InstanceType<typeof NScrollbar> | null>(null)

// 滚动到底部的函数
const scrollToBottom = async () => {
    await nextTick()
    if (scrollbarRef.value) {
        scrollbarRef.value.scrollTo({ top: 99999, behavior: 'smooth' })
    }
}

// 监听进度事件
onMounted(async () => {
    await getIds()
    await getMainJsIds()

    // 监听进度事件
    const unlistenFn = await listen<{ message: string }>('reset_progress', async (event) => {
        console.log('收到进度事件:', event)
        const msg = event.payload.message
        // 检查是否是国际化消息键
        const translatedMsg = msg.startsWith('authConfig.') ? t(msg) : msg
        progressMessage.value = translatedMsg
        progressMessages.value.push(translatedMsg)
        // 保持最新的 100 条消息
        if (progressMessages.value.length > 100) {
            progressMessages.value = progressMessages.value.slice(-100)
        }
        // 如果是错误消息，显示全局提示
        if (msg.includes('errors.')) {
            message.error(translatedMsg)
        } else if (msg.includes('progress.complete')) {
            message.success(translatedMsg)
        }
        // 自动滚动到底部
        await scrollToBottom()
    })

    // 组件卸载时取消监听
    onUnmounted(() => {
        unlistenFn()
    })
})
</script>

<style scoped>
.machine-id {
    --section-gap: 12px;
    --border-radius: 6px;
    max-width: 800px;
    margin: 0 auto;
}

.content-wrapper {
    display: flex;
    flex-direction: column;
}

.config-section,
.id-section,
.progress-section {
    background-color: v-bind('themeVars.cardColor');
    border-radius: var(--border-radius);
    padding-top: 12px;
}

.section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    margin-bottom: 8px;
    color: v-bind('themeVars.textColor2');
    font-weight: 500;
}

.id-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
    margin-top: 12px;
}

.id-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.id-label {
    font-size: 12px;
    color: v-bind('themeVars.textColor3');
    padding-left: 2px;
}

.progress-section {
    background-color: v-bind('themeVars.cardColor');
    border-radius: var(--border-radius);
}

:deep(.n-scrollbar) {
    height: 120px;
    border: 1px solid v-bind('themeVars.borderColor');
    border-radius: 3px;
}

.message-wrapper {
    padding: 6px 8px;
}

.message-item {
    display: block;
    padding: 2px 0;
    line-height: 1.4;
    font-size: 13px;
}

.actions {
    margin-top: 4px;
}

.action-buttons {
    display: flex;
    gap: 8px;
}

.action-btn {
    font-size: 12px;
    padding: 0 12px;
    height: 28px;
}

:deep(.n-input) {
    --n-height: 28px;
    --n-font-size: 12px;
}


:deep(.n-button) {
    --n-height: 28px;
    --n-font-size: 12px;
}

:deep(.n-scrollbar-rail) {
    z-index: 1;
}

:deep(.n-scrollbar-rail.n-scrollbar-rail--vertical) {
    right: 2px;
    top: 4px;
    bottom: 4px;
}

:deep(.n-scrollbar-rail.n-scrollbar-rail--horizontal) {
    left: 4px;
    right: 4px;
    bottom: 2px;
}

:deep(.n-scrollbar-content) {
    padding-right: 6px;
}

:deep(.n-card) {
    --n-padding-top: 0;
    --n-padding-bottom: 0;
    --n-padding-left: 0;
    --n-padding-right: 0;
}

:deep(.n-card__content) {
    padding: 0;
}
</style>