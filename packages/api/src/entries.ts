import Elysia, { t } from 'elysia'
import { ErrorSchema, Nullable } from './common'

const EntrySchema = t.Object(
	{
		id: t.String(),
		link: t.String({ format: 'uri' }),
		title: t.String(),
		publishedAt: Nullable(t.String({ format: 'date-time' })),
		description: Nullable(t.String()),
		author: Nullable(t.String()),
		thumbnailUrl: Nullable(t.String({ format: 'uri' })),
		hasRead: t.Boolean(),
		feedId: t.String(),
	},
	{
		$id: '#/components/schemas/Entry',
	},
)

export default new Elysia()
	.model({
		Entry: EntrySchema,
		Error: ErrorSchema,
	})
	.get(
		'/entries',
		async () => {
			return {
				hasMore: false,
				data: [],
			}
		},
		{
			type: 'application/json',
			response: {
				200: t.Object({
					hasMore: t.Boolean(),
					data: t.Array(t.Ref(EntrySchema)),
				}),
			},
		},
	)
