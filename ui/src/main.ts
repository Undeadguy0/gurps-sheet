import { createApp } from 'vue';
import './styles/fonts.css';
import './styles/themes.css';
import './styles/base.css';
import App from './App.vue';
import { applyTheme } from './theme';

applyTheme();
createApp(App).mount('#app');
