import type { FindOneParams } from '../common'
import type { Feed, FeedCreateData, FindManyFeedsParams } from './types'

export interface FeedsRepository {
	findMany(params: FindManyFeedsParams): Promise<Feed[]>

	findOne(params: FindOneParams): Promise<Feed | null>

	create(data: FeedCreateData): Promise<Feed>

	delete(params: FindOneParams): Promise<boolean>
}
