export class AppError extends Error {
	constructor(message?: string) {
		super(message ?? 'Internal server error')
	}
}

export class NotFoundError extends AppError {
	constructor(message?: string) {
		super(message ?? 'Resource not found')
	}
}

export class ConflictError extends AppError {
	constructor(message?: string) {
		super(message ?? 'Resource already exists')
	}
}
