<template>
    <n-card :title="t('apiConfig.title')" class="api-config" size="small">
        <n-space vertical size="small">
            <!-- API URL 配置区域 -->
            <n-form-item :label="t('apiConfig.urlLabel')" :show-feedback="false" size="small">
                <n-space :size="8" align="center">
                    <n-input v-model:value="apiUrl" type="text" :placeholder="t('apiConfig.urlLabel')"
                        :status="inputStatus" @input="validateUrl" size="small" style="height: 28px" />
                    <n-space :size="4">
                        <n-button type="primary" @click="testApi" :loading="isTestLoading"
                            :disabled="!isValidUrl || isLoading" size="small" style="padding: 0 12px; height: 28px">
                            {{ isTestLoading ? t('apiConfig.testing') : t('apiConfig.testApi') }}
                        </n-button>
                        <n-button type="success" @click="applyConfig" :loading="isApplyLoading"
                            :disabled="!canApply || isLoading" size="small" style="padding: 0 12px; height: 28px">
                            {{ isApplyLoading ? t('apiConfig.applying') : t('common.apply') }}
                        </n-button>
                        <n-button type="default" @click="resetToDefault" :disabled="isLoading" size="small"
                            style="padding: 0 12px; height: 28px">
                            {{ t('apiConfig.resetToDefault') }}
                        </n-button>
                    </n-space>
                </n-space>
            </n-form-item>

            <!-- API 返回格式示例和测试结果区域 -->
            <n-grid :cols="2" :x-gap="8">
                <n-grid-item>
                    <n-card :title="t('apiConfig.responseFormat')" size="small" :bordered="false" class="example-card">
                        <n-code :code="apiResponseExample" language="json" :word-wrap="true" show-line-numbers />
                    </n-card>
                </n-grid-item>
                <n-grid-item>
                    <n-card :title="t('apiConfig.testResponse')" size="small" :bordered="false" class="example-card">
                        <n-empty v-if="!testResponse" :description="t('apiConfig.noTestData')" size="tiny" />
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
import { useI18n } from 'vue-i18n'
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

defineOptions({
    name: 'ApiConfig'
})

const message = useMessage()
const { t } = useI18n()
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
        message.error(t('apiConfig.validUrl'))
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
        message.success(t('apiConfig.testSuccess'))
    } catch (error) {
        testResult.value = false
        message.error(t('apiConfig.testFailed') + ': ' + (error instanceof Error ? error.message : t('common.error')))
    } finally {
        isTestLoading.value = false
    }
}

async function applyConfig() {
    if (!canApply.value) return

    isApplyLoading.value = true

    try {
        await invoke('save_api_config', { url: apiUrl.value })
        message.success(t('apiConfig.applySuccess'))
    } catch (error) {
        message.error(t('apiConfig.applyFailed') + ': ' + (error instanceof Error ? error.message : t('common.error')))
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
            message.success(t('apiConfig.resetSuccess'))
            validateUrl()
        }
    } catch (error) {
        message.error(t('apiConfig.resetFailed') + ': ' + (error instanceof Error ? error.message : t('common.error')))
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
        message.error(t('apiConfig.loadFailed'))
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