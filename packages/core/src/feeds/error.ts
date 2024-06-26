import { NotFoundError } from '../common/error'

export class FeedNotFoundError extends NotFoundError {
	constructor(id: string) {
		super(`Feed not found with ID: ${id}`)
	}
}
