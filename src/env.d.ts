/// <reference types="vite/client" />
/// <reference types="vue/dist/vue.d.ts" />
/// <reference types="naive-ui/volar" />
/// <reference types="@vue/runtime-core" />

declare module "*.vue" {
  import type { DefineComponent } from "@vue/runtime-core";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
