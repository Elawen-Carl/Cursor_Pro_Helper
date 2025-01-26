import { createApp } from "vue";
import App from "./App.vue";
import naive from "naive-ui";
import i18n from "./locales";

// 导入 Naive UI 样式
import "vfonts/Lato.css";
import "vfonts/FiraCode.css";

const app = createApp(App);
app.use(naive);
app.use(i18n);
app.mount("#app");
