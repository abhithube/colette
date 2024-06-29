export class AppError extends Error {
	name = 'AppError'

	constructor(message?: string) {
		super(message ?? 'Internal server error')
	}
}

export class NotFoundError extends AppError {
	name = 'NotFoundError'

	constructor(message?: string) {
		super(message ?? 'Resource not found')
	}
}

export class ConflictError extends AppError {
	name = 'ConflictError'

	constructor(message?: string) {
		super(message ?? 'Resource already exists')
	}
}
