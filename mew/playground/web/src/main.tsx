import { createRoot } from 'react-dom/client';
import App from './App';
import './styles/main.css';

// Note: StrictMode removed to avoid double-invoking effects which causes WASM aliasing issues
createRoot(document.getElementById('root')!).render(<App />);
