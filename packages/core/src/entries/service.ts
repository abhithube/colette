import type { Session } from '../auth'
import { PAGINATION_LIMIT, type Paginated } from '../common'
import type { EntriesRepository } from './repository'
import type { Entry, ListParams } from './types'

export class EntriesService {
	constructor(private readonly repo: EntriesRepository) {}

	async list(params: ListParams, session: Session): Promise<Paginated<Entry>> {
		const entries = await this.repo.findMany({
			profileId: session.userId,
			limit: PAGINATION_LIMIT + 1,
			profileFeedId: params.feedId,
			publishedAt: params.publishedAt,
		})

		return {
			hasMore: entries.length > PAGINATION_LIMIT,
			data: entries.slice(0, PAGINATION_LIMIT),
		}
	}
}
