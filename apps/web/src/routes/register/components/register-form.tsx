import { useRegisterUserMutation } from '@colette/query'
import { Alert, Button, Card, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { getRouteApi, Link } from '@tanstack/react-router'
import { UserX } from 'lucide-react'
import { z } from 'zod'

const routeApi = getRouteApi('/register')

export const RegisterForm = () => {
  const context = routeApi.useRouteContext()
  const navigate = routeApi.useNavigate()

  const form = useForm({
    defaultValues: {
      email: '',
      password: '',
      passwordConfirm: '',
    },
    onSubmit: ({ value }) =>
      registerUser.mutate(
        {
          email: value.email,
          password: value.password,
        },
        {
          onSuccess: () => {
            form.reset()

            navigate({
              to: '/login',
              state: {
                registered: true,
              },
            })
          },
        },
      ),
  })

  const registerUser = useRegisterUserMutation(context.api)

  return (
    <>
      {registerUser.error && (
        <Alert.Root className="mb-4" variant="destructive">
          <UserX />
          <Alert.Title>Failed to log in</Alert.Title>
          <Alert.Description>{registerUser.error.message}</Alert.Description>
        </Alert.Root>
      )}
      <Card.Root>
        <Card.Header>
          <Card.Title>Register</Card.Title>
          <Card.Description>Register a new account</Card.Description>
        </Card.Header>
        <Card.Content>
          <form
            id="register"
            className="space-y-4"
            onSubmit={(e) => {
              e.preventDefault()
              form.handleSubmit()
            }}
          >
            <form.Field
              name="email"
              validators={{
                onBlur: z.string().email('Please enter a valid email'),
              }}
            >
              {(field) => {
                return (
                  <Field.Root invalid={field.state.meta.errors.length !== 0}>
                    <Field.Label>Email</Field.Label>
                    <Field.Input
                      type="email"
                      value={field.state.value}
                      placeholder="user@example.com"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>
                      {field.state.meta.errors[0]?.message}
                    </Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
            <form.Field
              name="password"
              validators={{
                onBlur: z
                  .string()
                  .min(8, 'Password must be at least 8 characters'),
              }}
            >
              {(field) => {
                return (
                  <Field.Root invalid={field.state.meta.errors.length !== 0}>
                    <Field.Label>Password</Field.Label>
                    <Field.Input
                      type="password"
                      value={field.state.value}
                      placeholder="********"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>
                      {field.state.meta.errors[0]?.message}
                    </Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
            <form.Field
              name="passwordConfirm"
              validators={{
                onBlur: () =>
                  form.getFieldValue('password') !==
                  form.getFieldValue('passwordConfirm')
                    ? 'Passwords do not match'
                    : undefined,
              }}
            >
              {(field) => {
                return (
                  <Field.Root invalid={field.state.meta.errors.length !== 0}>
                    <Field.Label>Confirm Password</Field.Label>
                    <Field.Input
                      type="password"
                      value={field.state.value}
                      placeholder="********"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>
                      {field.state.meta.errors[0]}
                    </Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
          </form>
        </Card.Content>
        <Card.Footer className="flex-col items-stretch gap-4">
          <Button form="register" disabled={registerUser.isPending}>
            Register
          </Button>
          <div className="self-center text-sm">
            Already have an account?{' '}
            <Link
              className="underline underline-offset-4"
              from="/register"
              to="/login"
            >
              Sign in
            </Link>
          </div>
        </Card.Footer>
      </Card.Root>
    </>
  )
}
