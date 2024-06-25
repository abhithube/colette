import type { FindOneParams } from '../common'
import type { Feed } from './model'

export interface FeedsRepository {
	findMany(params: FindManyFeedsParams): Promise<Feed[]>

	findOne(params: FindOneParams): Promise<Feed | null>

	delete(params: FindOneParams): Promise<boolean>
}

export type FindManyFeedsParams = {
	profileId: string
}
