import type { BookmarkScraped } from '@colette/core/types'
import { createBookmarkFormOptions } from '@colette/form'
import { useCreateBookmarkMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'

export const EditStep = (props: {
  formId: string
  bookmark: BookmarkScraped
  onClose: () => void
}) => {
  const form = useForm({
    ...createBookmarkFormOptions(props.bookmark),
    onSubmit: ({ value, formApi }) => {
      createBookmark.mutate(
        {
          url: props.bookmark.link,
          ...value,
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

  const createBookmark = useCreateBookmarkMutation()

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
  )
}
