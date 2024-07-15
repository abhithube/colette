import { NotFoundError } from '../common'

export class ProfileNotFoundError extends NotFoundError {
	name = 'ProfileNotFoundError'

	constructor(id: string) {
		super(`Profile not found with ID: ${id}`)
	}
}
