import { defineConfig } from '@pandacss/dev'

export default defineConfig({
	include: ['./src/**/*.{ts,tsx}'],
	jsxFramework: 'solid',
	preflight: true,
	presets: ['@pandacss/preset-base', '@park-ui/panda-preset'],
})
