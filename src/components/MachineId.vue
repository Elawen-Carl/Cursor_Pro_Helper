<template>
    <n-card class="machine-id" :bordered="false" size="small">
        <div class="content-wrapper">

            <div class="progress-section">
                <div class="section-title">
                    <n-icon size="14"><info-outlined /></n-icon>
                    进度消息
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
                        一键重置
                    </n-button>
                    <n-button @click="modifyId" :loading="modifyLoading" :disabled="modifyLoading" class="action-btn"
                        size="small">
                        <template #icon><edit-outlined /></template>
                        修改 ID
                    </n-button>
                    <n-button @click="backup" :loading="backupLoading" :disabled="backupLoading" class="action-btn"
                        size="small">
                        <template #icon><save-outlined /></template>
                        备份
                    </n-button>
                    <n-button @click="restore" :loading="restoreLoading" :disabled="restoreLoading" class="action-btn"
                        size="small">
                        <template #icon><rollback-outlined /></template>
                        还原
                    </n-button>
                </div>
            </div>
            <div class="config-section">
                <div class="section-title">
                    <n-icon size="14"><folder-outlined /></n-icon>
                    配置文件路径
                </div>
                <n-input readonly :value="configPath" placeholder="配置文件路径" :border="false" class="config-input"
                    size="small" />
            </div>


            <div class="id-section">
                <div class="section-title">
                    <n-icon size="14"><key-outlined /></n-icon>
                    当前 ID
                </div>
                <div class="id-grid">
                    <div class="id-item" v-for="(id, key) in ids" :key="key">
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

    // 监听进度事件
    const unlistenFn = await listen<{ message: string }>('reset_progress', async (event) => {
        console.log('收到进度事件:', event)
        progressMessage.value = event.payload.message
        progressMessages.value.push(event.payload.message)
        // 保持最新的 100 条消息
        if (progressMessages.value.length > 100) {
            progressMessages.value = progressMessages.value.slice(-100)
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
    --card-padding: 16px;
    --section-gap: 12px;
    --border-radius: 6px;
    max-width: 800px;
    margin: 0 auto;
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
    padding: 8px;
}

.section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    margin-bottom: 8px;
    color: v-bind('themeVars.textColor2');
}

.id-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
    padding: 0 4px;
}

.id-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.id-label {
    font-size: 11px;
    color: v-bind('themeVars.textColor3');
    padding-left: 2px;
}

.message-container {
    height: 120px;
    padding: 0;
    border: 1px solid v-bind('themeVars.borderColor');
    border-radius: 3px;
}

.message-wrapper {
    padding: 6px 8px;
}

.message-item {
    display: block;
    padding: 3px 0;
    line-height: 1.5;
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

:deep(.n-card) {
    --n-padding: var(--card-padding);
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
</style>