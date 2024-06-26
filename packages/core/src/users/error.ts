import { NotFoundError } from '../common/error'

export class UserNotFoundError extends NotFoundError {
	constructor(email: string) {
		super(`User not found with email: ${email}`)
	}
}
