import { type Session, UserNotAuthenticatedError } from '@colette/core'
import Elysia, { t } from 'elysia'
import { CookieSchema, ErrorSchema, Nullable } from './common'
import { lucia } from './deps'

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
	.guard({
		cookie: CookieSchema,
	})
	.derive(async (ctx) => {
		const cookie = ctx.cookie.auth_session.value
		if (!cookie) {
			throw new UserNotAuthenticatedError()
		}

		const sessionId = lucia.readSessionCookie(cookie)
		if (!sessionId) {
			throw new UserNotAuthenticatedError()
		}

		const { session: luciaSession } = await lucia.validateSession(sessionId)
		if (!luciaSession) {
			const sessionCookie = lucia.createBlankSessionCookie()
			ctx.cookie[sessionCookie.name].set({
				value: sessionCookie.value,
				...sessionCookie.attributes,
			})

			throw new UserNotAuthenticatedError()
		}
		if (luciaSession.fresh) {
			const sessionCookie = lucia.createSessionCookie(luciaSession.id)
			ctx.cookie[sessionCookie.name].set({
				value: sessionCookie.value,
				...sessionCookie.attributes,
			})
		}

		const session: Session = {
			userId: luciaSession.userId,
			profileId: luciaSession.profileId,
		}

		return {
			session,
		}
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
				200: t.Object({
					hasMore: t.Boolean(),
					data: t.Array(t.Ref(FeedSchema)),
				}),
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
			type: 'application/json',
			response: {
				200: 'Feed',
				404: 'Error',
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
			type: 'application/json',
			response: {
				204: t.Null({
					description: 'Feed deleted successfully',
				}),
				404: 'Error',
			},
		},
	)
