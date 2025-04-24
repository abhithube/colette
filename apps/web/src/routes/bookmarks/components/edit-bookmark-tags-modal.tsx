import type { BookmarkDetails } from '@colette/core'
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

      linkBookmarkTags.mutate(
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
        id="link-bookmark-tags"
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
        <Button form="link-bookmark-tags" disabled={linkBookmarkTags.isPending}>
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
