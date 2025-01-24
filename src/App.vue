<template>
    <n-config-provider :theme="currentTheme">
        <n-message-provider>
            <div class="app" :class="{ 'dark': isDark }">
                <n-layout class="layout">
                    <n-layout-header>
                        <div class="header">
                            <h1>Cursor Pro Helper</h1>
                            <n-button @click="toggleTheme" text>
                                {{ isDark ? '🌞' : '🌙' }}
                            </n-button>
                        </div>
                    </n-layout-header>
                    <n-layout-content class="content">
                        <ApiConfig />
                        <MachineId />
                    </n-layout-content>
                </n-layout>
            </div>
        </n-message-provider>
    </n-config-provider>
</template>

<script setup lang="ts">
import { ref, computed } from '@vue/runtime-core'
import type { GlobalTheme } from 'naive-ui'
import { darkTheme, NConfigProvider, NLayout, NLayoutHeader, NLayoutContent, NButton, NMessageProvider } from 'naive-ui'
import MachineId from './components/MachineId.vue'
import ApiConfig from './components/ApiConfig.vue'

const isDark = ref<boolean>(false)
const currentTheme = computed<GlobalTheme | null>(() => isDark.value ? darkTheme : null)

const toggleTheme = () => {
    isDark.value = !isDark.value
    // 同步更新 body 的主题类
    document.body.classList.toggle('dark', isDark.value)
}
</script>

<style>
/* 重置默认样式 */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

/* 定义主题变量 */
:root {
    --primary-bg: #ffffff;
    --secondary-bg: #f5f5f5;
    --border-color: #eee;
    --text-color: #333;
}

/* 暗色主题变量 */
:root.dark,
.dark {
    --primary-bg: #18181c;
    --secondary-bg: #1f1f23;
    --border-color: #333;
    --text-color: #fff;
}

/* 应用全局样式 */
body {
    background-color: var(--primary-bg);
    color: var(--text-color);
    transition: background-color 0.3s, color 0.3s;
}

.app {
    height: 100vh;
    background-color: var(--primary-bg);
}

.layout {
    background-color: var(--primary-bg) !important;
}

.header {
    padding: 10px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-color);
    background-color: var(--secondary-bg);
}

.content {
    padding: 20px;
    background-color: var(--primary-bg);
}

h1 {
    margin: 0;
    color: var(--text-color);
}

/* naive-ui 组件主题覆盖 */
.n-layout {
    color: var(--text-color);
}

.n-layout-header {
    background-color: var(--secondary-bg) !important;
}

.n-layout-content {
    background-color: var(--primary-bg) !important;
}
</style>