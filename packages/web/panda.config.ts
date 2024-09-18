import { customPreset } from '@colette/panda-preset'
import { defineConfig } from '@pandacss/dev'

export default defineConfig({
  presets: [customPreset],
  include: ['./src/**/*.{ts,tsx}'],
  lightningcss: true
})
