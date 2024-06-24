import type { User } from './model'

export interface UsersRepository {
	create(data: CreateData): Promise<User>
}

export type CreateData = {
	id: string
	email: string
	password: string
}
