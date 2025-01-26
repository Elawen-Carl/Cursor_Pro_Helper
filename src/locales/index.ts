import { createI18n } from "vue-i18n";
import en from "./en.json";
import zhCN from "./zh-CN.json";

// 获取浏览器语言设置
const getBrowserLanguage = () => {
  const language = navigator.language;
  if (language.startsWith("zh")) {
    return "zh-CN";
  }
  return "en";
};

// 获取存储的语言设置或使用浏览器语言
const getStoredLanguage = () => {
  return localStorage.getItem("language") || getBrowserLanguage();
};

export type SupportedLocale = "zh-CN" | "en";

// 创建 i18n 实例
const i18n = createI18n({
  legacy: false, // 使用组合式 API
  locale: getStoredLanguage() as SupportedLocale,
  fallbackLocale: "en",
  messages: {
    en: en,
    "zh-CN": zhCN,
  },
});

// 导出语言切换函数
export const setLanguage = (lang: SupportedLocale) => {
  i18n.global.locale.value = lang;
  localStorage.setItem("language", lang);
};

export default i18n;
