import { DrizzlePostgreSQLAdapter } from '@lucia-auth/adapter-drizzle'
import type { Database } from './client'
import { sessionsTable, usersTable } from './schema'

export function createAuthAdapter(db: Database) {
	return new DrizzlePostgreSQLAdapter(db, sessionsTable, usersTable)
}
