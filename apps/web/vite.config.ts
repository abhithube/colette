import path from 'node:path'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'
import react from '@vitejs/plugin-react-swc'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [
    react(),
    TanStackRouterVite({
      routesDirectory: path.join(
        __dirname,
        '../../node_modules/@colette/app/src/routes',
      ),
      generatedRouteTree: path.join(
        __dirname,
        '../../node_modules/@colette/app/src/routeTree.gen.ts',
      ),
      routeFileIgnorePrefix: '-',
      quoteStyle: 'single',
    }),
  ],
  resolve: {
    alias: {
      '~': path.resolve(__dirname, '../../packages/react-ui/src'),
    },
  },
})
