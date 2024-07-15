import { ConflictError, NotFoundError } from '../common'

export class UserNotFoundError extends NotFoundError {
	name = 'UserNotFoundError'

	constructor(email: string) {
		super(`User not found with email: ${email}`)
	}
}

export class UserAlreadyExistsError extends ConflictError {
	name = 'UserAlreadyExistsError'

	constructor(email: string) {
		super(`User already exists with email: ${email}`)
	}
}
