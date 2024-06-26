import type {
	Feed,
	FeedsRepository,
	FindManyFeedsParams,
	FindOneParams,
} from '@colette/core'
import type { Database } from '../client'
import {
	deleteProfileFeed,
	deleteProfileFeedEntries,
	selectProfileFeedById,
	selectProfileFeeds,
} from '../queries'

export class FeedsPostgresRepository implements FeedsRepository {
	constructor(private db: Database) {}

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
