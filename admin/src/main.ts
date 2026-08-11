import './assets/main_output.css'
import './assets/admin-list-layout.css'
import 'vuefinder/dist/style.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import { i18n } from './utils/i18n'
// import VueFinder from 'vuefinder/dist/vuefinder'

const app = createApp(App)
// app.use(VueFinder)


for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
    app.component(key, component)
}

app.use(createPinia())
app.use(i18n)
app.use(router)

app.mount('#app')
