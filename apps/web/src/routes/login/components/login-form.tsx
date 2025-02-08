import { useAPI } from '../../../lib/api-context'
import { loginOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { FC } from 'react'
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

export const LoginForm: FC = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  const form = useForm({
    defaultValues: {
      email: '',
      password: '',
    },
    onSubmit: ({ value }) => login(value),
  })

  const { mutateAsync: login, isPending } = useMutation(
    loginOptions(api, queryClient, {
      onSuccess: () => form.reset(),
    }),
  )

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <Card>
        <CardHeader>
          <CardTitle>Login</CardTitle>
          <CardDescription>Login to your account</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
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
        <CardFooter>
          <Button className="flex-1" disabled={isPending}>
            Login
          </Button>
        </CardFooter>
      </Card>
    </form>
  )
}
