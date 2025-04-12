import {
  useCreateBookmarkMutation,
  useScrapeBookmarkMutation,
} from '@colette/query'
import { Button, Dialog, Field } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { navigate } from 'wouter/use-browser-location'
import { z } from 'zod'

export const AddBookmarkModal = (props: { close: () => void }) => {
  const form = useForm({
    defaultValues: {
      url: '',
      title: '',
    },
    onSubmit: async ({ value }) => {
      const scraped = await scrapeBookmark.mutateAsync(value, {
        onSuccess: () => {
          form.reset()
          props.close()

          navigate('/bookmarks')
        },
      })

      createBookmark.mutate(
        {
          url: scraped.link,
          title: scraped.title,
          thumbnailUrl: scraped.thumbnailUrl,
          publishedAt: scraped.publishedAt,
          author: scraped.author,
        },
        {
          onSuccess: () => {
            form.reset()
            props.close()

            navigate('/stash')
          },
        },
      )
    },
  })

  const createBookmark = useCreateBookmarkMutation()

  const scrapeBookmark = useScrapeBookmarkMutation()

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title>Add Bookmark</Dialog.Title>
        <Dialog.Description>Add a bookmark to the stash.</Dialog.Description>
      </Dialog.Header>
      <form
        id="add-bookmark"
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <div className="mt-4 flex flex-col items-stretch space-y-4">
          <form.Field
            name="url"
            validators={{
              onBlur: z.string().url('Please enter a valid URL'),
            }}
          >
            {(field) => (
              <Field.Root className="space-y-2">
                <Field.Label>URL</Field.Label>
                <Field.Input
                  value={field.state.value}
                  placeholder="https://www.website.com"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <Field.HelperText>URL of the bookmark</Field.HelperText>
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
          <form.Field
            name="title"
            validators={{
              onSubmit: z.string().min(1, 'Title cannot be empty'),
            }}
          >
            {(field) => (
              <Field.Root className="space-y-2">
                <Field.Label>Title</Field.Label>
                <Field.Input
                  value={field.state.value}
                  onChange={(ev) => field.handleChange(ev.target.value)}
                />
                <Field.HelperText>Title of the bookmark</Field.HelperText>
                <Field.ErrorText>
                  {field.state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
        </div>
      </form>
      <Dialog.Footer>
        <Button
          form="add-bookmark"
          disabled={scrapeBookmark.isPending || createBookmark.isPending}
        >
          Submit
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
