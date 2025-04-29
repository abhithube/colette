import { Feed } from '@colette/core'
import { useCreateSubscriptionMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { getRouteApi } from '@tanstack/react-router'
import { z } from 'zod'

const routeApi = getRouteApi('/layout/subscriptions/')

export const EditStep = (props: {
  formId: string
  feed: Feed
  onClose: () => void
}) => {
  const context = routeApi.useRouteContext()

  const form = useForm({
    defaultValues: {
      title: props.feed.title,
      description: props.feed.description,
    },
    onSubmit: ({ value }) => {
      createSubscription.mutate(
        {
          title: value.title,
          description: value.description
            ? value.description.length > 0
              ? value.description
              : undefined
            : undefined,
          feedId: props.feed.id,
        },
        {
          onSuccess: () => {
            form.reset()
            props.onClose()
          },
        },
      )
    },
  })

  const createSubscription = useCreateSubscriptionMutation(context.api)

  return (
    <form
      id={props.formId}
      className="space-y-4"
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <form.Field
        name="title"
        validators={{
          onSubmit: z.string().min(1, 'Title cannot be empty'),
        }}
      >
        {(field) => {
          return (
            <Field.Root invalid={field.state.meta.errors.length !== 0}>
              <Field.Label>Title</Field.Label>
              <Field.Input
                value={field.state.value}
                onChange={(ev) => field.handleChange(ev.target.value)}
              />
              <Field.ErrorText>
                {field.state.meta.errors[0]?.message}
              </Field.ErrorText>
            </Field.Root>
          )
        }}
      </form.Field>

      <form.Field
        name="description"
        validators={{
          onSubmit: z.string().nullable(),
        }}
      >
        {(field) => {
          return (
            <Field.Root invalid={field.state.meta.errors.length !== 0}>
              <Field.Label>Description</Field.Label>
              <Field.Input
                value={field.state.value ?? undefined}
                onChange={(ev) => field.handleChange(ev.target.value)}
              />
              <Field.ErrorText>
                {field.state.meta.errors[0]?.message}
              </Field.ErrorText>
            </Field.Root>
          )
        }}
      </form.Field>
    </form>
  )
}
