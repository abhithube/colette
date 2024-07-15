import type { Session } from '../auth'
import type { Paginated } from '../common'
import type { PluginRegistry, ResponseParser, Scraper } from '../utils'
import type { HttpClient } from '../utils'
import { FeedNotFoundError } from './error'
import type { FeedsRepository } from './repository'
import type { ExtractedFeed, Feed, FeedDto, ProcessedFeed } from './types'

export class FeedsService {
	constructor(
		private readonly repo: FeedsRepository,
		private readonly http: HttpClient,
		private readonly parser: ResponseParser<Document>,
		private readonly registry: PluginRegistry<ExtractedFeed, ProcessedFeed>,
		private readonly defaultScraper: Scraper<ExtractedFeed, ProcessedFeed>,
	) {}

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

	async create(dto: FeedDto, session: Session): Promise<Feed> {
		const scraper =
			this.registry.load(new URL(dto.url).hostname) ?? this.defaultScraper

		const req = scraper.prepare(dto.url)

		const res = await this.http.get(req)
		const document = await this.parser.parse(res)

		const extracted = scraper.extract(dto.url, document)
		const processed = scraper.postprocess(dto.url, extracted)

		return this.repo.create({
			feedUrl: dto.url,
			processed,
			profileId: session.profileId,
		})
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
