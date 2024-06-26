import Elysia, { t } from 'elysia'
import { ErrorSchema, Nullable } from './common'

const FeedSchema = t.Object(
	{
		id: t.String(),
		link: t.String({ format: 'uri' }),
		title: t.String(),
		url: Nullable(t.String({ format: 'uri' })),
		customTitle: Nullable(t.String()),
		createdAt: t.String({ format: 'date-time' }),
		updatedAt: t.String({ format: 'date-time' }),
		unreadCount: t.Optional(t.Integer()),
	},
	{
		$id: '#/components/schemas/Feed',
	},
)

export default new Elysia()
	.model({
		Feed: FeedSchema,
		Error: ErrorSchema,
	})
	.get(
		'/feeds',
		async () => {
			return {
				hasMore: false,
				data: [],
			}
		},
		{
			type: 'application/json',
			response: {
				200: t.Object(
					{
						hasMore: t.Boolean(),
						data: t.Array(t.Ref(FeedSchema)),
					},
					{
						description: 'Paginated list of feeds',
					},
				),
			},
		},
	)
	.get(
		'/feeds/:id',
		async ({ params: { id } }) => {
			return {
				message: `Feed not found with id: ${id}`,
			}
		},
		{
			params: t.Object({
				id: t.String(),
			}),
			response: {
				200: {
					description: 'Feed by ID',
					...FeedSchema,
				},
				404: {
					description: 'Feed not found',
					...ErrorSchema,
				},
			},
		},
	)
	.delete(
		'/feeds/:id',
		async ({ params: { id } }) => {
			return {
				message: `Feed not found with id: ${id}`,
			}
		},
		{
			params: t.Object({
				id: t.String(),
			}),
			response: {
				204: t.Null({
					description: 'Feed deleted successfully',
				}),
				404: {
					description: 'Feed not found',
					...ErrorSchema,
				},
			},
		},
	)
