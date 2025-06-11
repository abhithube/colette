import { Bookmark } from '@colette/core/types'
import { UPDATE_BOOKMARK_FORM, updateBookmarkFormOptions } from '@colette/form'
import { useUpdateBookmarkMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useEffect } from 'react'

export const EditBookmarkModal = (props: {
  bookmark: Bookmark
  close: () => void
}) => {
  const form = useForm({
    ...updateBookmarkFormOptions(props.bookmark),
    onSubmit: ({ value, formApi }) => {
      if (
        value.title === props.bookmark.title &&
        value.thumbnailUrl === props.bookmark.thumbnailUrl &&
        value.publishedAt === props.bookmark.publishedAt &&
        value.author === props.bookmark.author
      ) {
        return props.close()
      }

      updateBookmark.mutate(
        {
          title: value.title,
          thumbnailUrl: value.thumbnailUrl,
          publishedAt: value.publishedAt,
          author: value.author
            ? value.author.length > 0
              ? value.author
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

  const updateBookmark = useUpdateBookmarkMutation(props.bookmark.id)

  useEffect(() => {
    form.reset()
  }, [form, props.bookmark.id])

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title className="line-clamp-1">
          Edit <span className="text-primary">{props.bookmark.title}</span>
        </Dialog.Title>
        <Dialog.Description>{"Edit a bookmark's metadata."}</Dialog.Description>
      </Dialog.Header>

      <form
        id={UPDATE_BOOKMARK_FORM}
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

        <form.Field name="thumbnailUrl">
          {(field) => {
            const errors = field.state.meta.errors

            return (
              <Field.Root invalid={errors.length !== 0}>
                <Field.Label>Thumbnail</Field.Label>
                <Field.Input
                  value={field.state.value ?? undefined}
                  onChange={(ev) => field.handleChange(ev.target.value)}
                />
                <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
              </Field.Root>
            )
          }}
        </form.Field>

        <form.Field name="publishedAt">
          {(field) => {
            const errors = field.state.meta.errors

            return (
              <Field.Root invalid={errors.length !== 0}>
                <Field.Label>Published At</Field.Label>
                <Field.Input
                  value={field.state.value ?? undefined}
                  onChange={(ev) => field.handleChange(ev.target.value)}
                />
                <Field.ErrorText>{errors[0]?.message}</Field.ErrorText>
              </Field.Root>
            )
          }}
        </form.Field>

        <form.Field name="author">
          {(field) => {
            const errors = field.state.meta.errors

            return (
              <Field.Root invalid={errors.length !== 0}>
                <Field.Label>Author</Field.Label>
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
        <Button form={UPDATE_BOOKMARK_FORM} disabled={updateBookmark.isPending}>
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
