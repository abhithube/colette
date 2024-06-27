import { NotFoundError } from '../common'

export class ProfileNotFoundError extends NotFoundError {
	constructor(id: string) {
		super(`Profile not found with ID: ${id}`)
	}
}
