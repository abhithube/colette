import { useRegisterUserMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import type { FC } from 'react'
import { Link } from 'wouter'
import { navigate } from 'wouter/use-browser-location'
import { z } from 'zod'
import { FormMessage } from '~/components/form'
import { Button } from '~/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'

export const RegisterForm: FC = () => {
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
    <form
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <Card>
        <CardHeader>
          <CardTitle>Register</CardTitle>
          <CardDescription>Register a new account</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {register.error && (
            <FormMessage>{register.error.message}</FormMessage>
          )}
          <form.Field
            name="email"
            validators={{
              onBlur: z.string().email('Please enter a valid email'),
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Label>Email</Label>
                <Input
                  type="email"
                  value={field.state.value}
                  placeholder="user@example.com"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
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
              <div className="space-y-1">
                <Label>Password</Label>
                <Input
                  type="password"
                  value={field.state.value}
                  placeholder="********"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
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
              <div className="space-y-1">
                <Label>Confirm Password</Label>
                <Input
                  type="password"
                  value={field.state.value}
                  placeholder="********"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
        </CardContent>
        <CardFooter className="flex-col items-stretch gap-4">
          <Button disabled={register.isPending}>Register</Button>
          <div className="self-center text-sm">
            Already have an account?{' '}
            <Link className="underline underline-offset-4" to="/login">
              Sign in
            </Link>
          </div>
        </CardFooter>
      </Card>
    </form>
  )
}
