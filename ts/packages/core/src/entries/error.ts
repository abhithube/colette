import { NotFoundError } from '../common'

export class EntryNotFoundError extends NotFoundError {
	name = 'EntryNotFoundError'

	constructor(id: string) {
		super(`Entry not found with ID: ${id}`)
	}
}
