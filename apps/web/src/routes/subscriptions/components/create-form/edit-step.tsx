import { Feed } from '@colette/core'
import { createSubscriptionFormOptions } from '@colette/form'
import { useCreateSubscriptionMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'

export const EditStep = (props: {
  formId: string
  feed: Feed
  onClose: () => void
}) => {
  const form = useForm({
    ...createSubscriptionFormOptions(props.feed),
    onSubmit: ({ value, formApi }) => {
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
            formApi.reset()

            props.onClose()
          },
        },
      )
    },
  })

  const createSubscription = useCreateSubscriptionMutation()

  return (
    <form
      id={props.formId}
      className="space-y-4"
      onSubmit={(e) => {
        e.preventDefault()
        form.handleSubmit()
      }}
    >
      <form.Field name="title">
        {(field) => {
          const errors = field.state.meta.errors

          return (
            <Field.Root invalid={errors.length !== 0}>
              <Field.Label>Title</Field.Label>
              <Field.Input
                value={field.state.value}
                onChange={(ev) => field.handleChange(ev.target.value)}
              />
              <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
            </Field.Root>
          )
        }}
      </form.Field>

      <form.Field name="description">
        {(field) => {
          const errors = field.state.meta.errors

          return (
            <Field.Root invalid={errors.length !== 0}>
              <Field.Label>Description</Field.Label>
              <Field.Input
                value={field.state.value ?? undefined}
                onChange={(ev) => field.handleChange(ev.target.value)}
              />
              <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
            </Field.Root>
          )
        }}
      </form.Field>
    </form>
  )
}
