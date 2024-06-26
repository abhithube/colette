import type { Session } from '../auth'
import type { Paginated } from '../common'
import { FeedNotFoundError } from './error'
import type { FeedsRepository } from './repository'
import type { Feed } from './types'

export class FeedsService {
	constructor(private repo: FeedsRepository) {}

	async list(session: Session): Promise<Paginated<Feed>> {
		const feeds = await this.repo.findMany({
			profileId: session.profileId,
		})

		return {
			hasMore: false,
			data: feeds,
		}
	}

	async get(id: string, session: Session): Promise<Feed> {
		const feed = await this.repo.findOne({
			id,
			profileId: session.profileId,
		})
		if (!feed) {
			throw new FeedNotFoundError(id)
		}

		return feed
	}

	async delete(id: string, session: Session): Promise<void> {
		const deleted = await this.repo.delete({
			id,
			profileId: session.profileId,
		})
		if (!deleted) {
			throw new FeedNotFoundError(id)
		}
	}
}
