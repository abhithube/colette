import { ConflictError, NotFoundError } from '../common'

export class UserNotFoundError extends NotFoundError {
	constructor(email: string) {
		super(`User not found with email: ${email}`)
	}
}

export class UserAlreadyExistsError extends ConflictError {
	constructor(email: string) {
		super(`User already exists with email: ${email}`)
	}
}
