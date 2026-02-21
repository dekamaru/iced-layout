import { defineConfig } from 'vitepress'
import type { DefaultTheme } from 'vitepress'
import { createRequire } from 'module'

const require = createRequire(import.meta.url)

let widgetSidebar: DefaultTheme.SidebarItem[] = []
let styleSidebar: DefaultTheme.SidebarItem[] = []
try {
  const generated = require('./generated-sidebar.json')
  widgetSidebar = generated.widgets ?? []
  styleSidebar = generated.styles ?? []
} catch {
  // Run `cargo run -p docs-gen` to generate pages and sidebar
}

export default defineConfig({
  srcDir: 'pages',
  ignoreDeadLinks: [/^\/schema\//, /^\/styles\//],
  title: 'iced-layout',
  description: 'XML layouts for the iced GUI framework',
  base: '/iced-layout/',
  themeConfig: {
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Schema', link: '/schema/container' },
      { text: 'Roadmap', link: '/roadmap' },
    ],
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'XML Format', link: '/guide/xml-format' },
          { text: 'State Access', link: '/guide/state-access' },
          { text: 'Loops and Conditions', link: '/guide/loops-and-conditions' },
          { text: 'Custom Widgets', link: '/guide/custom-widgets' },
          { text: 'Roadmap', link: '/roadmap' },
        ],
      },
      {
        text: 'Schema',
        items: widgetSidebar,
      },
      {
        text: 'Styles',
        items: styleSidebar,
      },
      {
        text: 'Reference',
        items: [
          { text: 'Types', link: '/types' },
        ],
      },
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/dekamaru/iced-layout' }],
    search: { provider: 'local' },
  }
})
