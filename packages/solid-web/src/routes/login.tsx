import { type SubmitHandler, createForm, valiForm } from '@modular-forms/solid'
import {
	type Navigator,
	action,
	useAction,
	useNavigate,
	useSubmission,
} from '@solidjs/router'
import { type Component, Show } from 'solid-js'
import { APIError } from '../lib/client'
import { type LoginDTO, authAPI, loginSchema } from '../lib/core/login'

const loginAction = action(async (data: LoginDTO, navigate: Navigator) => {
	try {
		const profile = await authAPI.login(data)

		navigate('/', {
			replace: true,
			state: profile,
		})
	} catch (error) {
		if (error instanceof APIError) {
			return error.message
		}

		throw new Error()
	}
})

export const Login: Component = () => {
	const navigate = useNavigate()

	const login = useAction(loginAction)
	const submission = useSubmission(loginAction)

	const [loginForm, { Form, Field }] = createForm<LoginDTO>({
		validate: valiForm(loginSchema),
	})

	const handleSubmit: SubmitHandler<LoginDTO> = async (values) => {
		await login(values, navigate)
	}

	return (
		<div>
			<h1>Login</h1>
			<Show when={submission.result}>
				<div>{submission.result}</div>
			</Show>
			<Form onSubmit={handleSubmit}>
				<Field name="email">
					{(field, props) => (
						<div>
							<label for={props.name}>Email: </label>
							<input {...props} id={props.name} type="email" required />
							{field.error && <div>{field.error}</div>}
						</div>
					)}
				</Field>
				<Field name="password">
					{(field, props) => (
						<div>
							<label for={props.name}>Password: </label>
							<input {...props} id={props.name} type="password" required />
							{field.error && <div>{field.error}</div>}
						</div>
					)}
				</Field>
				<button type="submit" disabled={loginForm.submitting}>
					Login
				</button>
			</Form>
		</div>
	)
}
