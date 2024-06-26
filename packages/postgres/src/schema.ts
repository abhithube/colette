import {
	boolean,
	integer,
	pgTable,
	serial,
	text,
	timestamp,
	unique,
} from 'drizzle-orm/pg-core'

export const feedsTable = pgTable('feeds', {
	id: serial('id').primaryKey(),
	link: text('link').notNull().unique(),
	title: text('title').notNull(),
	url: text('url'),
})

export const entriesTable = pgTable('entries', {
	id: serial('id').primaryKey(),
	link: text('link').notNull().unique(),
	title: text('title').notNull(),
	publishedAt: timestamp('published_at', { withTimezone: true }),
	description: text('description'),
	author: text('author'),
	thumbnailUrl: text('thumbnail_url'),
})

export const feedEntriesTable = pgTable(
	'feed_entries',
	{
		id: serial('id').primaryKey(),
		feedId: integer('feed_id')
			.notNull()
			.references(() => feedsTable.id, { onDelete: 'cascade' }),
		entryId: integer('entry_id')
			.notNull()
			.references(() => entriesTable.id, { onDelete: 'cascade' }),
	},
	(t) => ({
		feedIdEntryIdUnq: unique().on(t.feedId, t.entryId),
	}),
)

export const usersTable = pgTable('users', {
	id: text('id').primaryKey(),
	email: text('email').notNull().unique(),
	password: text('password').notNull(),
	createdAt: timestamp('created_at', { mode: 'string', withTimezone: true })
		.notNull()
		.defaultNow(),
	updatedAt: timestamp('updated_at', { mode: 'string', withTimezone: true })
		.notNull()
		.defaultNow(),
})

export const profilesTable = pgTable(
	'profiles',
	{
		id: text('id').primaryKey(),
		title: text('title').notNull(),
		imageUrl: text('image_url'),
		userId: text('user_id')
			.notNull()
			.references(() => usersTable.id, {
				onDelete: 'cascade',
			}),
		isDefault: boolean('is_default').notNull().default(false),
		createdAt: timestamp('created_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
		updatedAt: timestamp('updated_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
	},
	(t) => ({
		userIdIsDefaultUnq: unique().on(t.userId, t.isDefault),
	}),
)

export const sessionsTable = pgTable('sessions', {
	id: text('id').primaryKey(),
	expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
	userId: text('user_id')
		.notNull()
		.references(() => usersTable.id, { onDelete: 'cascade' }),
	profileId: text('profile_id')
		.notNull()
		.references(() => profilesTable.id, { onDelete: 'cascade' }),
})

export const profileFeedsTable = pgTable(
	'profile_feeds',
	{
		id: text('id').primaryKey(),
		customTitle: text('custom_title'),
		profileId: text('profile_id')
			.notNull()
			.references(() => profilesTable.id, { onDelete: 'cascade' }),
		feedId: integer('feed_id')
			.notNull()
			.references(() => feedsTable.id, { onDelete: 'restrict' }),
		createdAt: timestamp('created_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
		updatedAt: timestamp('updated_at', { mode: 'string', withTimezone: true })
			.notNull()
			.defaultNow(),
	},
	(t) => ({
		profileIdFeedIdUnq: unique().on(t.profileId, t.feedId),
	}),
)

export const profileFeedEntriesTable = pgTable(
	'profile_feed_entries',
	{
		id: text('id').primaryKey(),
		hasRead: boolean('has_read').notNull().default(false),
		profileFeedId: text('profile_feed_id')
			.notNull()
			.references(() => profileFeedsTable.id, { onDelete: 'cascade' }),
		feedEntryId: integer('feed_entry_id')
			.notNull()
			.references(() => feedEntriesTable.id, { onDelete: 'restrict' }),
	},
	(t) => ({
		profileFeedIdFeedEntryIdUnq: unique().on(t.profileFeedId, t.feedEntryId),
	}),
)
