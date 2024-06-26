import type { Entry, FindManyEntriesParams } from './types'

export interface EntriesRepository {
	findMany(params: FindManyEntriesParams): Promise<Entry[]>
}
