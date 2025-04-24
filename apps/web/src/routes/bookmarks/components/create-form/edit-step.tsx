import { BookmarkScraped } from '@colette/core'
import { useCreateBookmarkMutation } from '@colette/query'
import { Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { z } from 'zod'

export const EditStep = (props: {
  formId: string
  bookmark: BookmarkScraped
  onClose: () => void
}) => {
  const form = useForm({
    defaultValues: {
      title: props.bookmark.title,
      thumbnailUrl: props.bookmark.thumbnailUrl,
      publishedAt: props.bookmark.publishedAt,
      author: props.bookmark.author,
    },
    onSubmit: ({ value }) => {
      createBookmark.mutate(
        {
          url: props.bookmark.link,
          ...value,
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
        name="thumbnailUrl"
        validators={{
          onSubmit: z.string().url('URL is not valid').nullable(),
        }}
      >
        {(field) => {
          return (
            <Field.Root invalid={field.state.meta.errors.length !== 0}>
              <Field.Label>Thumbnail</Field.Label>
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

      <form.Field
        name="publishedAt"
        validators={{
          onSubmit: z.string().datetime('Date is not valid').nullable(),
        }}
      >
        {(field) => {
          return (
            <Field.Root invalid={field.state.meta.errors.length !== 0}>
              <Field.Label>Published At</Field.Label>
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

      <form.Field
        name="author"
        validators={{
          onSubmit: z.string().min(1, 'Author cannot be empty').nullable(),
        }}
      >
        {(field) => {
          return (
            <Field.Root invalid={field.state.meta.errors.length !== 0}>
              <Field.Label>Author</Field.Label>
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
