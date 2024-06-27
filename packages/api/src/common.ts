import { type Static, type TSchema, t } from 'elysia'

export const Nullable = <T extends TSchema>(schema: T) =>
	t.Unsafe<Static<T> | null>({
		...schema,
		nullable: true,
	})

export const ErrorSchema = t.Object(
	{
		message: t.String(),
	},
	{
		$id: '#/components/schemas/Error',
	},
)
