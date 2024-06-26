import { createDatabase } from '@colette/postgres'
import { NodePostgresAdapter } from '@lucia-auth/adapter-postgresql'
import { Lucia } from 'lucia'
import { Pool } from 'pg'

const pool = new Pool({
	connectionString: process.env.DATABASE_URL,
})

export const db = createDatabase(pool)

const adapter = new NodePostgresAdapter(pool, {
	user: 'users',
	session: 'sessions',
})

export const lucia = new Lucia(adapter, {
	sessionCookie: {
		attributes: {
			secure: process.env.NODE_ENV === 'production',
		},
	},
	getSessionAttributes: (attributes) => {
		return {
			profileId: attributes.profile_id,
		}
	},
})

declare module 'lucia' {
	interface Register {
		Lucia: typeof lucia
		DatabaseSessionAttributes: DatabaseSessionAttributes
	}

	interface DatabaseSessionAttributes {
		profile_id: string
	}
}
