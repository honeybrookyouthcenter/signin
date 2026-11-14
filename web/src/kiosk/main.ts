import { mount } from 'svelte';
import { api } from '../lib/api';
import '../lib/base.css';
import './kiosk.css';
import App from './App.svelte';
import { start } from './state.svelte';

start();

// Lock the staff page whenever the kiosk shows, so going from the staff page
// back to the kiosk needs the PIN again. This also runs when the browser's
// back button brings the kiosk back without loading it again.
window.addEventListener('pageshow', () => {
  void api('POST', '/api/admin/logout').catch(() => {});
});

mount(App, { target: document.getElementById('app')! });
