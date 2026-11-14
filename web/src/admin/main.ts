import { mount } from 'svelte';
import '../lib/base.css';
import './admin.css';
import App from './App.svelte';
import { startLive } from './live.svelte';

startLive();
mount(App, { target: document.getElementById('app')! });
