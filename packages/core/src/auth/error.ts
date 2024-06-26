import { AppError } from '../common'

export class UserNotAuthenticatedError extends AppError {
	constructor(message?: string) {
		super(message ?? 'User not authenticated')
	}
}
