export class ServerError extends Error {}

export class UnauthorizedError extends ServerError {}

export class ForbiddenError extends ServerError {}

export class NotFoundError extends ServerError {}

export class ConflictError extends ServerError {}

export class UnprocessableContentError extends ServerError {}

export class BadGatewayError extends ServerError {}
