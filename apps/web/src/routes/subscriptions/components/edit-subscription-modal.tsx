import type { SubscriptionDetails } from '@colette/core'
import { useUpdateSubscriptionMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useEffect } from 'react'
import { z } from 'zod'
import { TagsInput } from '~/components/tags-input'

export const EditSubscriptionModal = (props: {
  details: SubscriptionDetails
  close: () => void
}) => {
  const form = useForm({
    defaultValues: {
      title: props.details.subscription.title,
      tags: props.details.tags?.map((tag) => tag.id) ?? [],
    },
    onSubmit: ({ value }) => {
      let tags: string[] | undefined = value.tags
      if (props.details.tags) {
        const current = props.details.tags
        if (
          tags?.length === current.length &&
          tags.every((id) => current.find((tag) => tag.id === id) !== undefined)
        ) {
          tags = undefined
        }
      } else if (tags.length === 0) {
        tags = undefined
      }

      if (
        value.title === props.details.subscription.title &&
        tags === undefined
      ) {
        return props.close()
      }

      updateSubscription.mutate(
        {
          title: value.title,
          tags,
        },
        {
          onSuccess: () => {
            form.reset()
            props.close()
          },
        },
      )
    },
  })

  const updateSubscription = useUpdateSubscriptionMutation(
    props.details.subscription.id,
  )

  useEffect(() => {
    form.reset()
  }, [form, props.details.subscription.id])

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title className="line-clamp-1">
          Edit{' '}
          <span className="text-primary">
            {props.details.subscription.title}
          </span>
        </Dialog.Title>
        <Dialog.Description>
          {"Edit a subscription's metadata."}
        </Dialog.Description>
      </Dialog.Header>
      <form
        id="edit-subscription"
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <form.Field
          name="title"
          validators={{
            onBlur: z.string().min(1, "Title can't be empty"),
          }}
        >
          {(field) => {
            return (
              <Field.Root invalid={field.state.meta.errors.length !== 0}>
                <Field.Label>Title</Field.Label>
                <Field.Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.message}
                </Field.ErrorText>
              </Field.Root>
            )
          }}
        </form.Field>
        <form.Field name="tags">
          {(field) => {
            return (
              <Field.Root invalid={field.state.meta.errors.length !== 0}>
                <Field.Label>Tags</Field.Label>
                <TagsInput
                  state={field.state}
                  handleChange={field.handleChange}
                />
              </Field.Root>
            )
          }}
        </form.Field>
      </form>
      <Dialog.Footer>
        <Button
          form="edit-subscription"
          disabled={updateSubscription.isPending}
        >
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
