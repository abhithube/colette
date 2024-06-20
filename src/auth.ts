import { NodePostgresAdapter } from '@lucia-auth/adapter-postgresql'
import { Lucia } from 'lucia'
import { pool } from './db/client'

const adapter = new NodePostgresAdapter(pool, {
	user: 'users',
	session: 'sessions',
})

export const auth = new Lucia(adapter, {
	sessionCookie: {
		attributes: {
			secure: process.env.NODE_ENV === 'production',
		},
	},
})

declare module 'lucia' {
	interface Register {
		Lucia: typeof auth
	}
}
