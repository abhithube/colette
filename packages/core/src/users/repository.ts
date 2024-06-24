import type { User } from './model'

export interface UsersRepository {
	create(data: UserCreateData): Promise<User>
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
