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
		const rows = await selectProfileFeedById(this.db, params)
		if (rows.length === 0) {
			return null
		}

		return rows[0]
	}

	async delete(params: FindOneParams): Promise<boolean> {
		const feedResult = await deleteProfileFeed(this.db, params)
		if (feedResult.rowCount !== 1) {
			return false
		}

		await deleteProfileFeedEntries(this.db, {
			profileFeedId: params.id,
		})

		return true
	}
}
