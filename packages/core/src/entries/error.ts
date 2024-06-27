import { NotFoundError } from '../common'

export class EntryNotFoundError extends NotFoundError {
	constructor(id: string) {
		super(`Entry not found with ID: ${id}`)
	}
}
