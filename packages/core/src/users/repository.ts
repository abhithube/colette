import type { FindOneUserParams, User, UserCreateData } from './types'

export interface UsersRepository {
	findOne(params: FindOneUserParams): Promise<User | null>

	create(data: UserCreateData): Promise<User>
}
