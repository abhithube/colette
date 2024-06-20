import { type Kysely, sql } from 'kysely'

export async function up(db: Kysely<any>): Promise<void> {
	await db.schema
		.createTable('feeds')
		.addColumn('id', 'serial', (col) => col.primaryKey())
		.addColumn('link', 'text', (col) => col.notNull().unique())
		.addColumn('title', 'text', (col) => col.notNull())
		.addColumn('url', 'text')
		.execute()

	await db.schema
		.createTable('entries')
		.addColumn('id', 'serial', (col) => col.primaryKey())
		.addColumn('link', 'text', (col) => col.notNull().unique())
		.addColumn('title', 'text', (col) => col.notNull())
		.addColumn('published_at', 'timestamptz')
		.addColumn('description', 'text')
		.addColumn('author', 'text')
		.addColumn('thumbnail_url', 'text')
		.execute()

	await db.schema
		.createTable('feed_entries')
		.addColumn('id', 'serial', (col) => col.primaryKey())
		.addColumn('feed_id', 'integer', (col) =>
			col.notNull().references('feeds.id').onDelete('cascade'),
		)
		.addColumn('entry_id', 'integer', (col) =>
			col.notNull().references('entries.id').onDelete('cascade'),
		)
		.addUniqueConstraint('feed_entries_feed_id_entry_id_unq', [
			'feed_id',
			'entry_id',
		])
		.execute()

	await db.schema
		.createTable('users')
		.addColumn('id', 'text', (col) => col.primaryKey())
		.addColumn('email', 'text', (col) => col.notNull().unique())
		.addColumn('password', 'text', (col) => col.notNull())
		.addColumn('created_at', 'timestamptz', (col) =>
			col.notNull().defaultTo(sql`now()`),
		)
		.addColumn('updated_at', 'timestamptz', (col) =>
			col.notNull().defaultTo(sql`now()`),
		)
		.execute()

	await db.schema
		.createTable('sessions')
		.addColumn('id', 'text', (col) => col.primaryKey())
		.addColumn('expires_at', 'timestamptz', (col) => col.notNull())
		.addColumn('user_id', 'text', (col) =>
			col.notNull().references('users.id').onDelete('cascade'),
		)
		.execute()

	await db.schema
		.createTable('profiles')
		.addColumn('id', 'text', (col) => col.primaryKey())
		.addColumn('title', 'text', (col) => col.notNull())
		.addColumn('image_url', 'text')
		.addColumn('is_default', 'boolean', (col) => col.notNull().defaultTo(false))
		.addColumn('user_id', 'text', (col) =>
			col.notNull().references('users.id').onDelete('cascade'),
		)
		.addColumn('created_at', 'timestamptz', (col) =>
			col.notNull().defaultTo(sql`now()`),
		)
		.addColumn('updated_at', 'timestamptz', (col) =>
			col.notNull().defaultTo(sql`now()`),
		)
		.addUniqueConstraint('profiles_user_id_is_default_unq', [
			'user_id',
			'is_default',
		])
		.execute()
}

export async function down(db: Kysely<any>): Promise<void> {
	await db.schema.dropTable('profiles').execute()

	await db.schema.dropTable('sessions').execute()

	await db.schema.dropTable('users').execute()

	await db.schema.dropTable('feed_entries').execute()

	await db.schema.dropTable('entries').execute()

	await db.schema.dropTable('feeds').execute()
}
