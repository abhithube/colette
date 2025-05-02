import type { BookmarkDetails } from '@colette/core'
import {
  LINK_BOOKMARK_TAGS_FORM,
  linkBookmarkTagsFormOptions,
} from '@colette/form'
import { useLinkBookmarkTagsMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useEffect } from 'react'
import { TagsInput } from '~/components/tags-input'

export const EditBookmarkTagsModal = (props: {
  details: BookmarkDetails
  close: () => void
}) => {
  const form = useForm({
    ...linkBookmarkTagsFormOptions(props.details.tags),
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

      linkBookmarkTags.mutate(
        {
          tagIds,
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

  const linkBookmarkTags = useLinkBookmarkTagsMutation(
    props.details.bookmark.id,
  )

  useEffect(() => {
    form.reset()
  }, [form, props.details.bookmark.id])

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title className="line-clamp-1">
          Edit{' '}
          <span className="text-primary">{props.details.bookmark.title}</span>
        </Dialog.Title>
        <Dialog.Description>{"Edit a bookmark's tags."}</Dialog.Description>
      </Dialog.Header>

      <form
        id={LINK_BOOKMARK_TAGS_FORM}
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
          form={LINK_BOOKMARK_TAGS_FORM}
          disabled={linkBookmarkTags.isPending}
        >
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
