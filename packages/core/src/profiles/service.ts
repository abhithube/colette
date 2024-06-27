import type { Session } from '../auth'
import type { Paginated } from '../common'
import type { ValueGenerator } from '../utils'
import { ProfileNotFoundError } from './error'
import type { ProfilesRepository } from './repository'
import type { Profile, ProfileDto } from './types'

export class ProfilesService {
	constructor(
		private repo: ProfilesRepository,
		private idGenerator: ValueGenerator<string>,
	) {}

	async list(session: Session): Promise<Paginated<Profile>> {
		const profiles = await this.repo.findMany({
			userId: session.userId,
		})

		return {
			hasMore: false,
			data: profiles,
		}
	}

	async get(id: string, session: Session): Promise<Profile> {
		const profile = await this.repo.findOne({
			id,
			userId: session.userId,
		})
		if (!profile) {
			throw new ProfileNotFoundError(id)
		}

		return profile
	}

	async create(dto: ProfileDto, session: Session): Promise<Profile> {
		const profile = await this.repo.create({
			id: this.idGenerator.generate(),
			title: dto.title,
			imageUrl: dto.imageUrl,
			isDefault: false,
			userId: session.userId,
		})

		return profile
	}

	async delete(id: string, session: Session): Promise<void> {
		const deleted = await this.repo.delete({
			id,
			userId: session.userId,
		})
		if (!deleted) {
			throw new ProfileNotFoundError(id)
		}
	}
}
