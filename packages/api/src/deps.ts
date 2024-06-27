import {
	EntriesService,
	type ExtractedFeed,
	FeedsService,
	PluginRegistry,
	type ProcessedFeed,
	ProfilesService,
} from '@colette/core'
import { DefaultFeedScraper } from '@colette/plugins'
import {
	EntriesPostgresRepository,
	FeedsPostgresRepository,
	ProfilesPostgresRepository,
	createDatabase,
} from '@colette/postgres'
import { FetchClient, JSDOMParser, NanoidGenerator } from '@colette/utils'
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

const nanoidGenerator = new NanoidGenerator()
const fetchClient = new FetchClient()
const jsdomParser = new JSDOMParser()
const feedRegistry = new PluginRegistry<ExtractedFeed, ProcessedFeed>({})
const feedScraper = new DefaultFeedScraper()

const entriesRepository = new EntriesPostgresRepository(db)
const feedsRepository = new FeedsPostgresRepository(db, nanoidGenerator)
const profilesRepository = new ProfilesPostgresRepository(db, nanoidGenerator)

export const entriesService = new EntriesService(entriesRepository)
export const feedsService = new FeedsService(
	feedsRepository,
	fetchClient,
	jsdomParser,
	feedRegistry,
	feedScraper,
)
export const profilesService = new ProfilesService(profilesRepository)

declare module 'lucia' {
	interface Register {
		Lucia: typeof lucia
		DatabaseSessionAttributes: DatabaseSessionAttributes
	}

	interface DatabaseSessionAttributes {
		profile_id: string
	}
}
