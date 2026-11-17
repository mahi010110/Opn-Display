import { createRoot } from 'react-dom/client'

import './kit/styles/tokens/fonts.css'
import './kit/styles/tokens/colors.css'
import './kit/styles/tokens/typography.css'
import './kit/styles/tokens/spacing.css'
import './kit/styles/tokens/effects.css'
import './kit/styles/tokens/base.css'
import './kit/composants.css'
import './styles/icones.css'
import './styles/opn-display.css'

import { App } from './App'

const sombre = window.matchMedia('(prefers-color-scheme: dark)')
const appliquerLeTheme = () => {
  document.documentElement.dataset.theme = sombre.matches ? 'dark' : 'light'
}
appliquerLeTheme()
sombre.addEventListener('change', appliquerLeTheme)

// Pas de StrictMode : en développement, il lance deux fois les effets, donc
// deux installations en même temps.
createRoot(document.getElementById('racine')!).render(<App />)
