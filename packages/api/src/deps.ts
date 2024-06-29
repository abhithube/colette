import {
	AuthService,
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
	UsersPostgresRepository,
	createAuthAdapter,
	createDatabase,
} from '@colette/postgres'
import {
	ArgonHasher,
	FetchClient,
	JSDOMParser,
	NanoidGenerator,
} from '@colette/utils'
import { Lucia } from 'lucia'
import { Pool } from 'pg'

const pool = new Pool({
	connectionString: process.env.DATABASE_URL,
})

export const db = createDatabase(pool)

export const lucia = new Lucia(createAuthAdapter(db), {
	sessionCookie: {
		attributes: {
			secure: process.env.NODE_ENV === 'production',
		},
	},
	getSessionAttributes: (attributes) => {
		return {
			profileId: attributes.profileId,
		}
	},
})

const nanoidGenerator = new NanoidGenerator()
const argonHasher = new ArgonHasher()
const fetchClient = new FetchClient()
const jsdomParser = new JSDOMParser()
const feedRegistry = new PluginRegistry<ExtractedFeed, ProcessedFeed>({})
const feedScraper = new DefaultFeedScraper()

const entriesRepository = new EntriesPostgresRepository(db)
const feedsRepository = new FeedsPostgresRepository(db, nanoidGenerator)
const profilesRepository = new ProfilesPostgresRepository(db, nanoidGenerator)
const usersRepository = new UsersPostgresRepository(db, nanoidGenerator)

export const entriesService = new EntriesService(entriesRepository)
export const feedsService = new FeedsService(
	feedsRepository,
	fetchClient,
	jsdomParser,
	feedRegistry,
	feedScraper,
)
export const profilesService = new ProfilesService(profilesRepository)
export const authService = new AuthService(
	usersRepository,
	profilesRepository,
	argonHasher,
)

declare module 'lucia' {
	interface Register {
		Lucia: typeof lucia
		DatabaseSessionAttributes: DatabaseSessionAttributes
	}

	interface DatabaseSessionAttributes {
		profileId: string
	}
}
