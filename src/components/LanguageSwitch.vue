<template>
    <n-select v-model:value="currentLang" :options="languageOptions" size="small" style="width: 100px" />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { NSelect } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { setLanguage } from '../locales'
import type { SupportedLocale } from '../locales'

defineOptions({
    name: 'LanguageSwitch'
})

const { locale } = useI18n()
const currentLang = ref<SupportedLocale>(locale.value as SupportedLocale)

const languageOptions = [
    {
        label: '简体中文',
        value: 'zh-CN' as const
    },
    {
        label: 'English',
        value: 'en' as const
    }
]

watch(currentLang, (newLang) => {
    setLanguage(newLang)
})
</script>

<style scoped>
:deep(.n-select) {
    --n-height: 28px;
}
</style>