export type User = {
	id: string
	email: string
	password: string
	createdAt: string
	updatedAt: string
}

export type UserCreateData = {
	email: string
	password: string
}
