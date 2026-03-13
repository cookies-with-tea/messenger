import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './style.css'
import VueVirtualScroller from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(VueVirtualScroller)
app.mount('#app')
