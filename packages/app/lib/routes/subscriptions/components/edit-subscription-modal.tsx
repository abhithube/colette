import { Subscription } from '@colette/core/types'
import {
  UPDATE_SUBSCRIPTION_FORM,
  updateSubscriptionFormOptions,
} from '@colette/form'
import { useUpdateSubscriptionMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useEffect } from 'react'

export const EditSubscriptionModal = (props: {
  subscription: Subscription
  close: () => void
}) => {
  const form = useForm({
    ...updateSubscriptionFormOptions(props.subscription),
    onSubmit: ({ value, formApi }) => {
      if (
        value.title === props.subscription.title &&
        value.description === props.subscription.description
      ) {
        return props.close()
      }

      updateSubscription.mutate(
        {
          title: value.title,
          description: value.description
            ? value.description.length > 0
              ? value.description
              : undefined
            : undefined,
        },
        {
          onSuccess: () => {
            formApi.reset()

            props.close()
          },
        },
      )
    },
  })

  const updateSubscription = useUpdateSubscriptionMutation(
    props.subscription.id,
  )

  useEffect(() => {
    form.reset()
  }, [form, props.subscription.id])

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title className="line-clamp-1">
          Edit <span className="text-primary">{props.subscription.title}</span>
        </Dialog.Title>
        <Dialog.Description>
          {"Edit a subscription's metadata."}
        </Dialog.Description>
      </Dialog.Header>

      <form
        id={UPDATE_SUBSCRIPTION_FORM}
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
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
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

      <Dialog.Footer>
        <Button
          form={UPDATE_SUBSCRIPTION_FORM}
          disabled={updateSubscription.isPending}
        >
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
