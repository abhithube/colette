export type User = {
	id: string
	email: string
	password: string
	createdAt: Date
	updatedAt: Date
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
