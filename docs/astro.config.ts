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
      sidebar: [
        { label: 'Test', slug: 'test' },
        { label: 'Home', slug: 'introduction' },
        {
          label: 'Language',
          items: [
            'language/basics',
            'language/control-flow',
            'language/functions',
            'language/type-system',
            'language/classes',
            'language/interfaces',
            'language/enums',
            'language/modules',
            'language/error-handling',
            'language/pattern-matching',
            'language/decorators',
            'language/advanced-types',
            'language/operators',
          ],
        },
        {
          label: 'Guides',
          items: [
            'guides/migrating-from-lua',
            'guides/lua-targets',
          ],
        },
        {
          label: 'Reference',
          items: [
            'reference/cli',
            'reference/configuration',
            'reference/standard-library',
            'reference/utility-types',
            'reference/reflection',
            'reference/error-codes',
            'reference/grammar',
            'reference/keywords',
          ],
        },
        {
          label: 'Contributing',
          items: [
            'contributing/setup',
          ],
        },
      ],
    }),
  ],
  output: 'static',
});
