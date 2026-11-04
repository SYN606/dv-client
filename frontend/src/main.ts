import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

// Completely disable right-click context menu in any mode
window.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  e.stopPropagation();
  return false;
}, true);

document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  e.stopPropagation();
  return false;
}, true);

// Block keyboard context menu invocations (ContextMenu key, Shift+F10)
window.addEventListener('keydown', (e) => {
  if (e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10')) {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }
}, true);

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
