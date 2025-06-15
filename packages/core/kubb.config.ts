import { defineConfig } from '@kubb/core'
import { pluginClient } from '@kubb/plugin-client'
import { pluginOas } from '@kubb/plugin-oas'
import { pluginTs } from '@kubb/plugin-ts'
import { pluginZod } from '@kubb/plugin-zod'

export default defineConfig(() => {
  return {
    input: {
      path: '../../openapi.yaml',
    },
    output: {
      path: './lib/gen',
      clean: true,
      barrelType: false,
    },
    hooks: {
      done: 'npm run format',
    },
    plugins: [
      pluginOas({
        generators: [],
      }),
      pluginTs({
        output: {
          path: 'types.ts',
        },
      }),
      pluginZod({
        output: {
          path: 'schemas.ts',
        },
        typed: true,
      }),
      pluginClient({
        output: {
          path: 'http.ts',
        },
        parser: 'zod',
        importPath: '../client.ts',
      }),
    ],
  }
})
