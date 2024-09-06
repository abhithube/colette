import { defineConfig } from '@pandacss/dev'
import { createPreset } from '@park-ui/panda-preset'

export const customPreset = defineConfig({
  preflight: true,
  presets: ['@pandacss/preset-base', createPreset()],
  jsxFramework: 'react',
})
