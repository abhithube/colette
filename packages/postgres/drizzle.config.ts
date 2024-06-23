import { defineConfig } from 'drizzle-kit'

const databaseUrl = process.env.DATABASE_URL
if (!databaseUrl) {
	throw new Error('Missing DATABASE_URL environment variable')
}

export default defineConfig({
	dialect: 'postgresql',
	schema: './src/schema.ts',
	out: './drizzle',
	dbCredentials: {
		url: databaseUrl,
	},
})
