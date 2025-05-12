import { loginFormOptions, LOGIN_FORM } from '@colette/form'
import { useLoginUserMutation } from '@colette/query'
import { getRouteApi, Link, useRouter } from '@colette/router'
import { Alert, Card, Button, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { UserCheck, UserX } from 'lucide-react'

const routeApi = getRouteApi('/login')

export const LoginForm = () => {
  const router = useRouter()
  const search = routeApi.useSearch()
  const navigate = routeApi.useNavigate()

  const form = useForm({
    ...loginFormOptions(),
    onSubmit: ({ value, formApi }) => {
      navigate({
        replace: true,
        state: {},
      })

      loginUser.mutate(value, {
        onSuccess: () => {
          formApi.reset()

          if (search.from) {
            router.history.replace(search.from)
          } else {
            navigate({
              to: '/',
              replace: true,
            })
          }
        },
      })
    },
  })

  const loginUser = useLoginUserMutation()

  return (
    <>
      {router.state.location.state.registered && (
        <Alert.Root className="mb-4">
          <UserCheck />
          <Alert.Title>Registered!</Alert.Title>
          <Alert.Description>Your account has been created.</Alert.Description>
        </Alert.Root>
      )}
      {router.state.location.state.loggedOut && (
        <Alert.Root className="mb-4">
          <UserCheck />
          <Alert.Title>Logged out</Alert.Title>
          <Alert.Description>
            You have been logged out of your account.
          </Alert.Description>
        </Alert.Root>
      )}
      {loginUser.error && (
        <Alert.Root className="mb-4" variant="destructive">
          <UserX />
          <Alert.Title>Failed to log in</Alert.Title>
          <Alert.Description>{loginUser.error.message}</Alert.Description>
        </Alert.Root>
      )}
      <Card.Root>
        <Card.Header>
          <Card.Title>Login</Card.Title>
          <Card.Description>Login to your account</Card.Description>
        </Card.Header>
        <Card.Content>
          <form
            id={LOGIN_FORM}
            className="space-y-4"
            onSubmit={(e) => {
              e.preventDefault()
              form.handleSubmit()
            }}
          >
            <form.Field name="email">
              {(field) => {
                const errors = field.state.meta.errors

                return (
                  <Field.Root invalid={errors.length !== 0}>
                    <Field.Label>Email</Field.Label>
                    <Field.Input
                      type="email"
                      value={field.state.value}
                      placeholder="user@example.com"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
            <form.Field name="password">
              {(field) => {
                const errors = field.state.meta.errors

                return (
                  <Field.Root invalid={errors.length !== 0}>
                    <Field.Label>Password</Field.Label>
                    <Field.Input
                      type="password"
                      value={field.state.value}
                      placeholder="********"
                      onChange={(e) => field.handleChange(e.target.value)}
                      onBlur={field.handleBlur}
                    />
                    <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
                  </Field.Root>
                )
              }}
            </form.Field>
          </form>
        </Card.Content>
        <Card.Footer className="flex-col items-stretch gap-4">
          <Button form={LOGIN_FORM} disabled={loginUser.isPending}>
            Login
          </Button>
          <div className="self-center text-sm">
            {"Don't have an account? "}
            <Link
              className="underline underline-offset-4"
              from="/login"
              to="/register"
            >
              Sign up
            </Link>
          </div>
        </Card.Footer>
      </Card.Root>
    </>
  )
}
