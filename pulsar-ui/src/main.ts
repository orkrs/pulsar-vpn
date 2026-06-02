import { createApp } from 'vue'
import Tres from '@tresjs/core' // <-- ИМПОРТ 3D ДВИЖКА
import './style.css'
import App from './App.vue'

const app = createApp(App)

// Подключаем TresJS к нашему приложению (БЕЗ ЭТОГО 3D НЕ РАБОТАЕТ)
app.use(Tres)

app.mount('#app')