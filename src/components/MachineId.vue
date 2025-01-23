<template>
    <n-card class="machine-id" :bordered="false">
        <div class="content-wrapper">
            <div class="config-section">
                <div class="section-title">
                    <n-icon><folder-outlined /></n-icon>
                    配置文件路径
                </div>
                <n-input readonly :value="configPath" placeholder="配置文件路径" :border="false" class="config-input" />
            </div>

            <div class="id-section">
                <div class="section-title">
                    <n-icon><key-outlined /></n-icon>
                    当前 ID
                </div>
                <div class="id-grid">
                    <div class="id-item" v-for="(id, key) in ids" :key="key">
                        <div class="id-label">{{ labels[key] }}</div>
                        <n-input readonly :value="id" :placeholder="labels[key]" :border="false" class="id-input" />
                    </div>
                </div>
            </div>

            <div class="progress-section">
                <div class="section-title">
                    <n-icon><info-outlined /></n-icon>
                    进度消息
                </div>
                <div class="message-container">
                    <div v-for="(msg, index) in progressMessages" :key="index" class="message-item"
                        :class="{ 'latest-message': index === progressMessages.length - 1 }">
                        {{ msg }}
                    </div>
                </div>
            </div>

            <div class="actions">
                <div class="action-buttons">
                    <n-button type="primary" @click="resetId" :loading="resetLoading" class="action-btn">
                        <template #icon><reload-outlined /></template>
                        一键重置
                    </n-button>
                    <n-button @click="modifyId" :loading="modifyLoading" :disabled="modifyLoading" class="action-btn">
                        <template #icon><edit-outlined /></template>
                        修改 ID
                    </n-button>
                    <n-button @click="backup" :loading="backupLoading" :disabled="backupLoading" class="action-btn">
                        <template #icon><save-outlined /></template>
                        备份
                    </n-button>
                    <n-button @click="restore" :loading="restoreLoading" :disabled="restoreLoading" class="action-btn">
                        <template #icon><rollback-outlined /></template>
                        还原
                    </n-button>
                </div>
            </div>
        </div>
    </n-card>
</template>

<script setup lang="ts">

import { ref, onMounted, computed, type Ref, onUnmounted } from '@vue/runtime-core'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
    NButton,
    NInput,
    NCard,
    NIcon,
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

const message = useMessage()
const themeVars = useThemeVars()

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
const machineId = ref<string>('')
const macMachineId = ref<string>('')
const devDeviceId = ref<string>('')
const sqmId = ref<string>('')

const ids = computed<IdsMap>(() => ({
    machineId: machineId.value,
    macMachineId: macMachineId.value,
    devDeviceId: devDeviceId.value,
    sqmId: sqmId.value
}))

const labels = {
    machineId: 'Machine ID',
    macMachineId: 'Mac Machine ID',
    devDeviceId: 'Dev Device ID',
    sqmId: 'SQM ID'
} as const
//todo-可以设置获取账号的api
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
        message.error('获取ID信息失败: ' + e)
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
        message.success(successMessage)
    } catch (e) {
        console.error(errorMessage, e)
        message.error(`${errorMessage}: ${e}`)
    } finally {
        loadingRef.value = false
    }
}

// 重置 ID
const resetId = () => executeAction(
    () => invoke('reset_machine_id'),
    resetLoading,
    '重置机器ID成功',
    '重置机器ID失败'
)

// 修改 ID
const modifyId = () => executeAction(
    () => invoke('update_machine_id'),
    modifyLoading,
    '修改机器ID成功',
    '修改机器ID失败'
)

// 备份配置
const backup = () => executeAction(
    () => invoke('backup_config'),
    backupLoading,
    '备份成功',
    '备份失败',
    false
)

// 还原配置
const restore = () => executeAction(
    () => invoke('restore_config'),
    restoreLoading,
    '还原成功',
    '还原失败'
)

// 监听进度事件
onMounted(async () => {
    await getIds()

    // 监听进度事件
    const unlistenFn = await listen<{ message: string }>('reset_progress', (event) => {
        console.log('收到进度事件:', event)
        progressMessage.value = event.payload.message
        progressMessages.value.push(event.payload.message)
        // 保持最新的 100 条消息
        if (progressMessages.value.length > 100) {
            progressMessages.value = progressMessages.value.slice(-100)
        }
    })

    // 组件卸载时取消监听
    onUnmounted(() => {
        unlistenFn()
    })
})
</script>

<style scoped>
.machine-id {
    --card-padding: 24px;
    --section-gap: 20px;
    --border-radius: 8px;
}

.content-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--section-gap);
}

.config-section,
.id-section,
.progress-section {
    background-color: v-bind('themeVars.cardColor');
    border-radius: var(--border-radius);
}

.section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: v-bind('themeVars.textColor3');
    margin-bottom: 12px;
}

.id-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
}

.id-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.id-label {
    font-size: 12px;
    color: v-bind('themeVars.textColor3');
    padding-left: 4px;
}

.actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: v-bind('themeVars.cardColor');
    border-radius: var(--border-radius);
    padding: 16px;
}

.action-buttons {
    display: flex;
    gap: 12px;
}

.action-btn {
    display: flex;
    align-items: center;
    gap: 4px;
}

:deep(.n-card) {
    background-color: v-bind('themeVars.bodyColor');
}

:deep(.n-input) {
    background-color: v-bind('themeVars.inputColor');
}

:deep(.n-button:not(.n-button--primary)) {
    background-color: v-bind('themeVars.buttonColor2');
    border-color: v-bind('themeVars.buttonColor2');
    color: v-bind('themeVars.textColor2');
}

:deep(.n-button:not(.n-button--primary):hover) {
    background-color: v-bind('themeVars.buttonColor2Hover');
    border-color: v-bind('themeVars.buttonColor2Hover');
}

:deep(.n-input .n-input__input-el) {
    cursor: default;
    user-select: all;
}

:deep(.n-checkbox) {
    --n-text-color: v-bind('themeVars.textColor2');
}

.progress-section {
    max-height: 200px;
    display: flex;
    flex-direction: column;
}

.message-container {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    background-color: v-bind('themeVars.inputColor');
    border-radius: 4px;
    max-height: 150px;
}

.message-item {
    padding: 4px 8px;
    font-size: 13px;
    line-height: 1.4;
    color: v-bind('themeVars.textColor2');
    border-radius: 2px;
}

.message-item:not(:last-child) {
    margin-bottom: 4px;
}

.latest-message {
    color: v-bind('themeVars.textColor1');
    background-color: v-bind('themeVars.primaryColorSuppl');
}

/* 自定义滚动条样式 */
.message-container::-webkit-scrollbar {
    width: 6px;
}

.message-container::-webkit-scrollbar-track {
    background: transparent;
}

.message-container::-webkit-scrollbar-thumb {
    background-color: v-bind('themeVars.scrollbarColor');
    border-radius: 3px;
}

.message-container::-webkit-scrollbar-thumb:hover {
    background-color: v-bind('themeVars.scrollbarColorHover');
}
</style>