import { AppError } from '../common'

export class UserNotAuthenticatedError extends AppError {
	name = 'UserNotAuthenticatedError'

	constructor(message?: string) {
		super(message ?? 'User not authenticated')
	}
}
