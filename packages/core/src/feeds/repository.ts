import type { FindOneParams } from '../common'
import type { Feed, FindManyFeedsParams } from './types'

export interface FeedsRepository {
	findMany(params: FindManyFeedsParams): Promise<Feed[]>

	findOne(params: FindOneParams): Promise<Feed | null>

	delete(params: FindOneParams): Promise<boolean>
}
