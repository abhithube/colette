import type { Entry } from './model'

export interface EntriesRepository {
	findMany(params: FindManyEntriesParams): Promise<Entry[]>
}

export type FindManyEntriesParams = {
	profileId: string
	publishedAt?: Date
	profileFeedId?: string
}
