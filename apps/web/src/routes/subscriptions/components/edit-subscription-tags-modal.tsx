import type { SubscriptionDetails } from '@colette/core'
import {
  LINK_SUBSCRIPTION_TAGS_FORM,
  linkSubscriptionTagsFormOptions,
} from '@colette/form'
import { useLinkSubscriptionTagsMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { getRouteApi } from '@tanstack/react-router'
import { useEffect } from 'react'
import { TagsInput } from '~/components/tags-input'

const routeApi = getRouteApi('/layout/subscriptions/')

export const EditSubscriptionTagsModal = (props: {
  details: SubscriptionDetails
  close: () => void
}) => {
  const context = routeApi.useRouteContext()

  const form = useForm({
    ...linkSubscriptionTagsFormOptions(),
    onSubmit: ({ value, formApi }) => {
      let tagIds: string[] | undefined = value.tagIds
      if (props.details.tags) {
        const current = props.details.tags
        if (
          tagIds?.length === current.length &&
          tagIds.every(
            (id) => current.find((tag) => tag.id === id) !== undefined,
          )
        ) {
          tagIds = undefined
        }
      } else if (tagIds.length === 0) {
        tagIds = undefined
      }

      if (tagIds === undefined) {
        return props.close()
      }

      linkSubscriptionTags.mutate(
        {
          tagIds: tagIds,
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

  const linkSubscriptionTags = useLinkSubscriptionTagsMutation(
    context.api,
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
        id={LINK_SUBSCRIPTION_TAGS_FORM}
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <form.Field name="tagIds">
          {(field) => {
            const errors = field.state.meta.errors

            return (
              <Field.Root invalid={errors.length !== 0}>
                <Field.Label>Tags</Field.Label>
                <TagsInput
                  api={context.api}
                  state={field.state}
                  handleChange={field.handleChange}
                />
                <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
              </Field.Root>
            )
          }}
        </form.Field>
      </form>

      <Dialog.Footer>
        <Button
          form={LINK_SUBSCRIPTION_TAGS_FORM}
          disabled={linkSubscriptionTags.isPending}
        >
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
