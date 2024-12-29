import { loginOptions } from '@colette/solid-query'
import { createForm } from '@tanstack/solid-form'
import { createMutation, useQueryClient } from '@tanstack/solid-query'
import type { Component } from 'solid-js'
import { z } from 'zod'
import { useAPI } from '../lib/api-context'
import { Button, Card, Field } from '../lib/components'

export const LoginForm: Component = () => {
  const form = createForm(() => ({
    defaultValues: {
      email: '',
      password: '',
    },
    onSubmit: ({ value }) => login(value),
  }))

  const { mutateAsync: login, isPending } = createMutation(() =>
    loginOptions(
      {
        onSuccess: async (user) => {
          console.log(user)

          form.reset()
        },
      },
      useAPI(),
      useQueryClient(),
    ),
  )

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <Card.Root>
        <Card.Header class="space-y-2">
          <Card.Title>Login</Card.Title>
          <Card.Description>Login to your account</Card.Description>
        </Card.Header>
        <Card.Content class="space-y-4">
          <form.Field
            name="email"
            validators={{
              onBlur: z.string().email('Email is not valid'),
            }}
          >
            {(field) => (
              <Field.Root invalid={field().state.meta.errors.length > 0}>
                <Field.Label>Email</Field.Label>
                <Field.Input
                  placeholder="user@email.com"
                  onChange={(e) => field().handleChange(e.target.value)}
                  onBlur={() => field().handleBlur()}
                />
                <Field.ErrorText>
                  {field().state.meta.errors[0]?.toString()}
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
              <Field.Root invalid={field().state.meta.errors.length > 0}>
                <Field.Label>Password</Field.Label>
                <Field.Input
                  type="password"
                  placeholder="********"
                  onChange={(e) => field().handleChange(e.target.value)}
                  onBlur={() => field().handleBlur()}
                />
                <Field.ErrorText>
                  {field().state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
        </Card.Content>
        <Card.Footer>
          <Button class="grow" disabled={isPending}>
            Login
          </Button>
        </Card.Footer>
      </Card.Root>
    </form>
  )
}
