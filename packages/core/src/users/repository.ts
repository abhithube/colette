import type { User, UserCreateData } from './types'

export interface UsersRepository {
	create(data: UserCreateData): Promise<User>
}
