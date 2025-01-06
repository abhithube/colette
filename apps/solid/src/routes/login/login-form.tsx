import { getActiveOptions, loginOptions } from '@colette/query'
import { useNavigate } from '@solidjs/router'
import { createForm } from '@tanstack/solid-form'
import {
  createMutation,
  createQuery,
  useQueryClient,
} from '@tanstack/solid-query'
import { type Component, createEffect } from 'solid-js'
import { z } from 'zod'
import { Button } from '~/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import {
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '~/components/ui/text-field'
import { useAPI } from '~/lib/api-context'

export const LoginForm: Component = () => {
  const api = useAPI()
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const query = createQuery(() => ({
    ...getActiveOptions(api),
    retry: false,
  }))

  createEffect(() => {
    if (!query.isLoading && query.data) {
      navigate('/', {
        replace: true,
      })
    }
  })

  const form = createForm(() => ({
    defaultValues: {
      email: '',
      password: '',
    },
    onSubmit: ({ value }) => mutation.mutate(value),
  }))

  const mutation = createMutation(() =>
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
        <CardHeader class="space-y-2">
          <CardTitle>Login</CardTitle>
          <CardDescription>Login to your account</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <form.Field
            name="email"
            validators={{
              onBlur: z.string().email('Email is not valid'),
            }}
          >
            {(field) => (
              <TextField
                class="space-y-1"
                value={field().state.value}
                onChange={field().handleChange}
                onBlur={field().handleBlur}
                validationState={
                  field().state.meta.errors.length > 0 ? 'invalid' : 'valid'
                }
              >
                <TextFieldLabel>Email</TextFieldLabel>
                <TextFieldInput placeholder="user@email.com" />
                <TextFieldErrorMessage>
                  {field().state.meta.errors[0]?.toString()}
                </TextFieldErrorMessage>
              </TextField>
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
              <TextField
                class="space-y-1"
                value={field().state.value}
                onChange={field().handleChange}
                onBlur={field().handleBlur}
                validationState={
                  field().state.meta.errors.length > 0 ? 'invalid' : 'valid'
                }
              >
                <TextFieldLabel>Password</TextFieldLabel>
                <TextFieldInput type="password" placeholder="********" />
                <TextFieldErrorMessage>
                  {field().state.meta.errors[0]?.toString()}
                </TextFieldErrorMessage>
              </TextField>
            )}
          </form.Field>
        </CardContent>
        <CardFooter>
          <Button class="grow" type="submit" disabled={mutation.isPending}>
            Login
          </Button>
        </CardFooter>
      </Card>
    </form>
  )
}
