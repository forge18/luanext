import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://forge18.github.io',
  base: '/luanext',
  integrations: [
    starlight({
      title: 'LuaNext',
      description: 'A typed superset of Lua with gradual typing, inspired by TypeScript',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/forge18/luanext' },
      ],
    }),
  ],
  output: 'static',
});
