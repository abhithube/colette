import { type Session, UserNotAuthenticatedError } from '@colette/core'
import Elysia from 'elysia'
import { lucia } from '../deps'

export default new Elysia().derive({ as: 'scoped' }, async (ctx) => {
	const cookieHeader = ctx.request.headers.get('Cookie')
	if (!cookieHeader) {
		throw new UserNotAuthenticatedError()
	}

	const sessionId = lucia.readSessionCookie(cookieHeader)
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