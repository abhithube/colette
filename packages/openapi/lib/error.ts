import type { ValidationError } from './types'

export class BaseError extends Error {}

export class UnprocessableContentError extends Error {
	errors: Record<string, ValidationError[]>

	constructor(errors: Record<string, ValidationError[]>) {
		super()

		this.errors = errors
	}
}

export class UnauthorizedError extends BaseError {}

export class NotFoundError extends BaseError {}

export class BadGatewayError extends BaseError {}
