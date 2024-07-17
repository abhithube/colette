export class APIError extends Error {}

export class UnauthorizedError extends APIError {}

export class NotFoundError extends APIError {}

export class ConflictError extends APIError {}

export class UnprocessableContentError extends APIError {}

export class BadGatewayError extends APIError {}
