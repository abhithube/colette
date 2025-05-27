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
          path: 'zod.ts',
        },
      }),
      pluginClient({
        output: {
          path: 'client.ts',
        },
        parser: 'zod',
        importPath: '../fetch.ts',
      }),
    ],
  }
})
