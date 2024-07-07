import { type SubmitHandler, createForm, valiForm } from '@modular-forms/solid'
import { action, useAction, useNavigate, useSubmission } from '@solidjs/router'
import { type Component, Show } from 'solid-js'
import * as v from 'valibot'

const loginSchema = v.object({
	email: v.pipe(
		v.string(),
		v.nonEmpty('Please enter your email.'),
		v.email('The email provided is not valid.'),
	),
	password: v.pipe(
		v.string(),
		v.nonEmpty('Please enter a password.'),
		v.minLength(8, 'Your password must have at least 8 characters.'),
	),
})

type LoginData = v.InferInput<typeof loginSchema>

const login = action(async (data: LoginData) => {
	const message: string | undefined = 'error'

	await new Promise((resolve) => setTimeout(resolve, 1000))

	return message
})

export const Login: Component = () => {
	const navigate = useNavigate()

	const doLogin = useAction(login)
	const submission = useSubmission(login)

	const [loginForm, { Form, Field }] = createForm<LoginData>({
		validate: valiForm(loginSchema),
	})

	const handleSubmit: SubmitHandler<LoginData> = async (values) => {
		const error = await doLogin(values)
		if (!error) {
			navigate('/', {
				replace: true,
			})
		}
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
