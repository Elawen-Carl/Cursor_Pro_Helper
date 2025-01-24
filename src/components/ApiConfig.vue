<template>
    <n-card title="API 配置" class="api-config" size="small">
        <n-space vertical size="small">
            <!-- API URL 配置区域 -->
            <n-form-item label="API URL" :show-feedback="false" size="small">
                <n-space :size="8" align="center">
                    <n-input v-model:value="apiUrl" type="text" placeholder="请输入API URL" :status="inputStatus"
                        @input="validateUrl" size="small" style="height: 28px" />
                    <n-space :size="4">
                        <n-button type="primary" @click="testApi" :loading="isTestLoading"
                            :disabled="!isValidUrl || isLoading" size="small" style="padding: 0 12px; height: 28px">
                            {{ isTestLoading ? '测试中...' : '测试API' }}
                        </n-button>
                        <n-button type="success" @click="applyConfig" :loading="isApplyLoading"
                            :disabled="!canApply || isLoading" size="small" style="padding: 0 12px; height: 28px">
                            {{ isApplyLoading ? '应用中...' : '应用' }}
                        </n-button>
                        <n-button type="default" @click="resetToDefault" :disabled="isLoading" size="small"
                            style="padding: 0 12px; height: 28px">
                            重置
                        </n-button>
                    </n-space>
                </n-space>
            </n-form-item>

            <!-- API 返回格式示例和测试结果区域 -->
            <n-grid :cols="2" :x-gap="8">
                <n-grid-item>
                    <n-card title="API 返回格式示例" size="small" :bordered="false" class="example-card">
                        <n-code :code="apiResponseExample" language="json" :word-wrap="true" show-line-numbers />
                    </n-card>
                </n-grid-item>
                <n-grid-item>
                    <n-card title="测试接口返回数据" size="small" :bordered="false" class="example-card">
                        <n-empty v-if="!testResponse" description="暂无测试数据" size="tiny" />
                        <n-code v-else :code="testResponse" language="json" :word-wrap="true" show-line-numbers />
                    </n-card>
                </n-grid-item>
            </n-grid>
        </n-space>
    </n-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
    NCard,
    NSpace,
    NInput,
    NButton,
    NCode,
    NFormItem,
    NGrid,
    NGridItem,
    NEmpty,
    useMessage,
} from 'naive-ui'

const message = useMessage()
const DEFAULT_API_URL = 'https://cursor-account-api.vercel.app/account/random'

const apiUrl = ref(DEFAULT_API_URL)
const isValidUrl = ref(true)
const isTestLoading = ref(false)
const isApplyLoading = ref(false)
const testResult = ref<boolean | null>(null)
const testResponse = ref<string>('')

const apiResponseExample = `{
  "success": true,
  "data": {
    "email": "example@email.com",  // 账号邮箱（必需）
    "token": "your_token_here"     // 认证令牌（必需）
  },
  "message": "Success"
}`

const isLoading = computed(() => isTestLoading.value || isApplyLoading.value)
const canApply = computed(() => isValidUrl.value && testResult.value === true)

const inputStatus = computed(() => {
    if (!isValidUrl.value) return 'error'
    if (testResult.value === true) return 'success'
    if (testResult.value === false) return 'error'
    return undefined
})

function validateUrl() {
    try {
        new URL(apiUrl.value)
        isValidUrl.value = true
    } catch {
        isValidUrl.value = false
        message.error('请输入有效的URL')
    }
}

async function testApi() {
    if (!isValidUrl.value) return

    isTestLoading.value = true
    testResponse.value = ''

    try {
        const response = await fetch(apiUrl.value)
        const data = await response.json()

        // 保存测试响应
        testResponse.value = JSON.stringify(data, null, 2)

        // 验证响应格式（只检查必需字段）
        if (!data.success || !data.data || !data.data.email || !data.data.token) {
            throw new Error('API响应格式不正确，必须包含 email 和 token 字段')
        }

        testResult.value = true
        message.success('API测试成功')
    } catch (error) {
        testResult.value = false
        message.error(`API测试失败: ${error instanceof Error ? error.message : '未知错误'}`)
    } finally {
        isTestLoading.value = false
    }
}

async function applyConfig() {
    if (!canApply.value) return

    isApplyLoading.value = true

    try {
        await invoke('save_api_config', { url: apiUrl.value })
        message.success('配置已成功应用')
    } catch (error) {
        message.error(`应用配置失败: ${error instanceof Error ? error.message : '未知错误'}`)
    } finally {
        isApplyLoading.value = false
    }
}

async function resetToDefault() {
    try {
        const result = await invoke('reset_api_config')
        const config = result as { url: string }
        if (config && 'url' in config) {
            apiUrl.value = config.url
            testResult.value = null
            testResponse.value = ''
            message.success('已重置为默认API URL')
            validateUrl()
        }
    } catch (error) {
        message.error(`重置失败: ${error instanceof Error ? error.message : '未知错误'}`)
    }
}

// 加载当前配置
onMounted(async () => {
    try {
        const result = await invoke('get_api_config')
        const config = result as { url: string }
        if (config && 'url' in config) {
            apiUrl.value = config.url
            validateUrl()
        }
    } catch (error) {
        console.error('Failed to load API config:', error)
        message.error('加载配置失败')
    }
})
</script>

<style scoped>
.api-config {
    max-width: 800px;
    margin: 0 auto;
}

.example-card {
    height: 100%;
    background-color: var(--n-card-color);
}

:deep(.n-code) {
    max-height: 200px;
    overflow: auto;
    font-size: 11px;
    line-height: 1.2;
}

:deep(.n-code-line) {
    padding: 0;
}

:deep(.n-input) {
    min-width: 300px;
}

:deep(.n-card-header) {
    padding: 6px 10px;
    font-size: 11px;
}

:deep(.n-card__content) {
    padding: 0 10px 10px;
}

:deep(.n-form-item) {
    margin-bottom: 0;
}

:deep(.n-form-item-label) {
    font-size: 12px;
    height: 24px;
    line-height: 24px;
}

:deep(.n-empty) {
    font-size: 11px;
    padding: 8px 0;
}

:deep(.n-button) {
    font-size: 12px;
}

:deep(.n-input__input) {
    font-size: 12px;
    height: 28px;
}

:deep(.n-code .line-numbers) {
    padding: 0 4px;
    font-size: 10px;
}

:deep(.n-code-line) {
    padding: 0 4px;
}

:deep(.n-card-header__main) {
    font-size: 14px;
    font-weight: 500;
}

:deep(.n-card) {
    --n-padding-top: 0;
    --n-padding-bottom: 0;
    --n-padding-left: 0;
    --n-padding-right: 0;
}


:deep(.n-form-item) {
    margin-bottom: 8px;
}

:deep(.n-form-item-label) {
    font-size: 12px;
    height: 24px;
    line-height: 24px;
}
</style>