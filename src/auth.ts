import { NodePostgresAdapter } from '@lucia-auth/adapter-postgresql'
import { Lucia } from 'lucia'
import type { Client, Pool } from 'pg'

export function createAuth(client: Client | Pool) {
	const adapter = new NodePostgresAdapter(client, {
		user: 'users',
		session: 'sessions',
	})

	return new Lucia(adapter, {
		sessionCookie: {
			attributes: {
				secure: process.env.NODE_ENV === 'production',
			},
		},
	})
}

declare module 'lucia' {
	interface Register {
		Lucia: ReturnType<typeof createAuth>
	}
}
