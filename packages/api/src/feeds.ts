import Elysia, { t } from 'elysia'
import { ErrorSchema, Nullable } from './common'
import { feedsService } from './deps'
import session from './plugins/session'

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
	.decorate({
		feedsService,
	})
	.use(session)
	.get('/feeds', (ctx) => ctx.feedsService.list(ctx.session), {
		type: 'application/json',
		response: {
			200: t.Object({
				hasMore: t.Boolean(),
				data: t.Array(t.Ref(FeedSchema)),
			}),
		},
	})
	.get(
		'/feeds/:id',
		(ctx) => ctx.feedsService.get(ctx.params.id, ctx.session),
		{
			params: t.Object({
				id: t.String(),
			}),
			type: 'application/json',
			response: {
				200: 'Feed',
				404: 'Error',
			},
		},
	)
	.delete(
		'/feeds/:id',
		(ctx) => ctx.feedsService.delete(ctx.params.id, ctx.session),
		{
			params: t.Object({
				id: t.String(),
			}),
			type: 'application/json',
			response: {
				204: t.Void({
					description: 'Feed deleted successfully',
				}),
				404: 'Error',
			},
		},
	)
