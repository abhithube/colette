import type {
	Feed,
	FeedCreateData,
	FeedsRepository,
	FindManyFeedsParams,
	FindOneParams,
	ValueGenerator,
} from '@colette/core'
import type { Database } from '../client'
import {
	deleteProfileFeed,
	deleteProfileFeedEntries,
	insertEntry,
	insertFeed,
	insertFeedEntry,
	insertProfileFeed,
	insertProfileFeedEntry,
	selectProfileFeedById,
	selectProfileFeeds,
} from '../queries'

export class FeedsPostgresRepository implements FeedsRepository {
	constructor(
		private readonly db: Database,
		private readonly idGenerator: ValueGenerator<string>,
	) {}

	async findMany(data: FindManyFeedsParams): Promise<Feed[]> {
		return selectProfileFeeds(this.db, data)
	}

	async findOne(params: FindOneParams): Promise<Feed | null> {
		const [feed] = await selectProfileFeedById(this.db, params)
		if (!feed) {
			return null
		}

		return feed
	}

	async create(data: FeedCreateData): Promise<Feed> {
		const profileFeedId = await this.db.transaction(async (tx) => {
			const [feed] = await insertFeed(tx, {
				link: data.processed.link.href,
				title: data.processed.title,
				url:
					data.feedUrl === data.processed.link.href ? undefined : data.feedUrl,
			})
			if (!feed) {
				throw new Error('Feed not created')
			}

			const [profileFeed] = await insertProfileFeed(tx, {
				id: this.idGenerator.generate(),
				profileId: data.profileId,
				feedId: feed.id,
			})
			if (!profileFeed) {
				throw new Error('Profile feed not created')
			}

			for (const e of data.processed.entries) {
				const [entry] = await insertEntry(tx, {
					link: e.link.href,
					title: e.title,
					publishedAt: e.published?.toISOString(),
					description: e.description,
					author: e.author,
					thumbnailUrl: e.thumbnail?.href,
				})
				if (!entry) {
					throw new Error('Entry not created')
				}

				const [feedEntry] = await insertFeedEntry(tx, {
					feedId: feed.id,
					entryId: entry.id,
				})
				if (!feedEntry) {
					throw new Error('Feed entry not created')
				}

				const [profileFeedEntry] = await insertProfileFeedEntry(tx, {
					id: this.idGenerator.generate(),
					profileFeedId: profileFeed.id,
					feedEntryId: feedEntry.id,
				})
				if (!profileFeedEntry) {
					throw new Error('Profile feed entry not created')
				}
			}

			return profileFeed.id
		})

		const feed = await this.findOne({
			id: profileFeedId,
			profileId: data.profileId,
		})
		if (!feed) {
			throw new Error('Profile feed not created')
		}

		return feed
	}

	async delete(params: FindOneParams): Promise<boolean> {
		return this.db.transaction(async (tx) => {
			const result = await deleteProfileFeed(tx, params)
			if (result.rowCount !== 1) {
				return false
			}

			await deleteProfileFeedEntries(tx, {
				profileFeedId: params.id,
			})

			return true
		})
	}
}
