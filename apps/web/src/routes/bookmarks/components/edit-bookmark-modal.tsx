import { Bookmark } from '@colette/core'
import { useUpdateBookmarkMutation } from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { getRouteApi } from '@tanstack/react-router'
import { useEffect } from 'react'
import { z } from 'zod'

const routeApi = getRouteApi('/layout/stash')

export const EditBookmarkModal = (props: {
  bookmark: Bookmark
  close: () => void
}) => {
  const context = routeApi.useRouteContext()

  const form = useForm({
    defaultValues: {
      title: props.bookmark.title,
      thumbnailUrl: props.bookmark.thumbnailUrl,
      publishedAt: props.bookmark.publishedAt,
      author: props.bookmark.author,
    },
    onSubmit: ({ value }) => {
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
            form.reset()
            props.close()
          },
        },
      )
    },
  })

  const updateBookmark = useUpdateBookmarkMutation(
    context.api,
    props.bookmark.id,
  )

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
        id="edit-bookmark"
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

      <Dialog.Footer>
        <Button form="edit-bookmark" disabled={updateBookmark.isPending}>
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
