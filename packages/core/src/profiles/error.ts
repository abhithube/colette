import { NotFoundError } from '../common/error'

export class ProfileNotFoundError extends NotFoundError {
	constructor(id: string) {
		super(`Profile not found with ID: ${id}`)
	}
}
