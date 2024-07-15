import type { PasswordHasher } from '@colette/core'

export class ArgonHasher implements PasswordHasher {
	hash(password: string): Promise<string> {
		return Bun.password.hash(password)
	}

	verify(password: string, hashed: string): Promise<boolean> {
		return Bun.password.verify(password, hashed)
	}
}
