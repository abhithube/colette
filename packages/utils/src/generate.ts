import type { ValueGenerator } from '@colette/core'
import { nanoid } from 'nanoid'

export class NanoidGenerator implements ValueGenerator<string> {
	generate(): string {
		return nanoid()
	}
}
