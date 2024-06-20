import type { Kysely } from 'kysely'

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
		.execute()
}

export async function down(db: Kysely<any>): Promise<void> {
	await db.schema.dropTable('feed_entries').execute()

	await db.schema.dropTable('entries').execute()

	await db.schema.dropTable('feeds').execute()
}
