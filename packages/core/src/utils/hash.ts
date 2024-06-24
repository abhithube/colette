export interface PasswordHasher {
	hash(password: string): Promise<string>

	verify(password: string, hashed: string): Promise<boolean>
}