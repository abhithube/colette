import type { Component } from 'solid-js'
import { Button, Card, Field } from '../lib/components'
import { createForm } from '@tanstack/solid-form'
import { z } from 'zod'

export const LoginForm: Component = () => {
  const form = createForm(() => ({
    defaultValues: {
      email: '',
      password: '',
    },
    onSubmit: ({ value }) => {
      console.log(value)
    },
  }))

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
          <Button class="grow">Login</Button>
        </Card.Footer>
      </Card.Root>
    </form>
  )
}
