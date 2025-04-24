import type { SubscriptionDetails } from '@colette/core'
import { useLinkSubscriptionTagsMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useEffect } from 'react'
import { TagsInput } from '~/components/tags-input'

export const EditSubscriptionTagsModal = (props: {
  details: SubscriptionDetails
  close: () => void
}) => {
  const form = useForm({
    defaultValues: {
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

      if (tags === undefined) {
        return props.close()
      }

      linkSubscriptionTags.mutate(
        {
          tagIds: tags,
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

  const linkSubscriptionTags = useLinkSubscriptionTagsMutation(
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
        <Dialog.Description>{"Edit a subscription's tags."}</Dialog.Description>
      </Dialog.Header>

      <form
        id="link-subscription-tags"
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
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
          form="link-subscription-tags"
          disabled={linkSubscriptionTags.isPending}
        >
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
