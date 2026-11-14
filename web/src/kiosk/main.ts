import { mount } from 'svelte';
import '../lib/base.css';
import './kiosk.css';
import App from './App.svelte';
import { start } from './state.svelte';

start();
mount(App, { target: document.getElementById('app')! });
