import type {
	EntriesRepository,
	Entry,
	FindManyEntriesParams,
} from '@colette/core'
import type { Database } from '../client'
import { selectProfileFeedEntries } from '../queries'

export class EntrysPostgresRepository implements EntriesRepository {
	constructor(private readonly db: Database) {}

	async findMany(data: FindManyEntriesParams): Promise<Entry[]> {
		return selectProfileFeedEntries(this.db, data)
	}
}
