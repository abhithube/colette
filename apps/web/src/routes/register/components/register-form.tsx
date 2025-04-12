import { useRegisterUserMutation } from '@colette/query'
import { Button, Card, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { Link } from 'wouter'
import { navigate } from 'wouter/use-browser-location'
import { z } from 'zod'

export const RegisterForm = () => {
  const form = useForm({
    defaultValues: {
      email: '',
      password: '',
      passwordConfirm: '',
    },
    onSubmit: ({ value }) =>
      register.mutate(
        {
          email: value.email,
          password: value.password,
        },
        {
          onSuccess: () => {
            form.reset()

            navigate('/login', {
              state: {
                registered: true,
              },
            })
          },
        },
      ),
  })

  const register = useRegisterUserMutation()

  return (
    <Card.Root>
      <Card.Header>
        <Card.Title>Register</Card.Title>
        <Card.Description>Register a new account</Card.Description>
      </Card.Header>
      <Card.Content>
        {register.error && (
          <Field.ErrorText className="mb-2">
            {register.error.message}
          </Field.ErrorText>
        )}
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
            {(field) => (
              <Field.Root className="space-y-2">
                <Field.Label>Email</Field.Label>
                <Field.Input
                  type="email"
                  value={field.state.value}
                  placeholder="user@example.com"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
          <form.Field
            name="password"
            validators={{
              onBlur: z
                .string()
                .min(8, 'Password must be at least 8 characters'),
            }}
          >
            {(field) => (
              <Field.Root className="space-y-2">
                <Field.Label>Password</Field.Label>
                <Field.Input
                  type="password"
                  value={field.state.value}
                  placeholder="********"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
          <form.Field
            name="passwordConfirm"
            validators={{
              onBlur: () =>
                form.getFieldValue('password') !==
                  form.getFieldValue('passwordConfirm') &&
                'Passwords do not match',
            }}
          >
            {(field) => (
              <Field.Root className="space-y-2">
                <Field.Label>Confirm Password</Field.Label>
                <Field.Input
                  type="password"
                  value={field.state.value}
                  placeholder="********"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
        </form>
      </Card.Content>
      <Card.Footer className="flex-col items-stretch gap-4">
        <Button form="register" disabled={register.isPending}>
          Register
        </Button>
        <div className="self-center text-sm">
          Already have an account?{' '}
          <Link className="underline underline-offset-4" to="/login">
            Sign in
          </Link>
        </div>
      </Card.Footer>
    </Card.Root>
  )
}
