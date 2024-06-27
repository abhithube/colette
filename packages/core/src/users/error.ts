import { NotFoundError } from '../common'

export class UserNotFoundError extends NotFoundError {
	constructor(email: string) {
		super(`User not found with email: ${email}`)
	}
}
