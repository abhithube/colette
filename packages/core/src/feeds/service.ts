import type { Session } from '../auth'
import { FeedNotFoundError } from './error'
import type { FeedsRepository } from './repository'

export class FeedsService {
	constructor(private repo: FeedsRepository) {}

	async list(session: Session) {
		return this.repo.findMany({
			profileId: session.profileId,
		})
	}

	async get(id: string, session: Session) {
		const feed = await this.repo.findOne({
			id,
			profileId: session.profileId,
		})
		if (!feed) {
			throw new FeedNotFoundError(id)
		}

		return feed
	}

	async delete(id: string, session: Session) {
		const deleted = await this.repo.delete({
			id,
			profileId: session.profileId,
		})
		if (!deleted) {
			throw new FeedNotFoundError(id)
		}
	}
}
