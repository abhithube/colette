export type User = {
	id: string
	email: string
	password: string
	createdAt: string
	updatedAt: string
}

export type UserCreateData = {
	user: {
		id: string
		email: string
		password: string
	}
	profile: {
		id: string
	}
}
