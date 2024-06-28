import type { Profile, ProfilesRepository } from '../profiles'
import {
	UserAlreadyExistsError,
	UserNotFoundError,
	type UsersRepository,
} from '../users'
import type { PasswordHasher } from '../utils'
import { UserNotAuthenticatedError } from './error'
import type { AuthDto } from './types'

export class AuthService {
	constructor(
		private readonly usersRepo: UsersRepository,
		private readonly profilesRepo: ProfilesRepository,
		private readonly hasher: PasswordHasher,
	) {}

	async login(dto: AuthDto): Promise<Profile> {
		const user = await this.usersRepo.findOne({
			email: dto.email,
		})
		if (!user) {
			throw new UserNotFoundError(dto.email)
		}

		const valid = await this.hasher.verify(dto.password, user.password)
		if (!valid) {
			throw new UserNotAuthenticatedError()
		}

		const profile = await this.profilesRepo.findOne({
			userId: user.id,
		})
		if (!profile) {
			throw new Error('Profile not created')
		}

		return profile
	}

	async register(dto: AuthDto): Promise<Profile> {
		const hashed = await this.hasher.hash(dto.password)

		try {
			const user = await this.usersRepo.create({
				email: dto.email,
				password: hashed,
			})

			const profile = await this.profilesRepo.findOne({
				userId: user.id,
			})
			if (!profile) {
				throw new Error('Profile not created')
			}

			return profile
		} catch (error) {
			throw new UserAlreadyExistsError(dto.email)
		}
	}
}
