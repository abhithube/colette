import {
  useCreateBookmarkMutation,
  useScrapeBookmarkMutation,
} from '@colette/query'
import { useForm } from '@tanstack/react-form'
import type { FC } from 'react'
import { navigate } from 'wouter/use-browser-location'
import { z } from 'zod'
import { FormDescription, FormMessage } from '~/components/form'
import { Button } from '~/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Input } from '~/components/ui/input'
import { Label } from '~/components/ui/label'

export const AddBookmarkModal: FC<{ close: () => void }> = (props) => {
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
    <DialogContent className="p-6">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <DialogHeader>
          <DialogTitle>Add Bookmark</DialogTitle>
          <DialogDescription>Add a bookmark to the stash.</DialogDescription>
        </DialogHeader>
        <div className="mt-4 flex flex-col items-stretch space-y-4">
          <form.Field
            name="url"
            validators={{
              onBlur: z.string().url('Please enter a valid URL'),
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Label>URL</Label>
                <Input
                  value={field.state.value}
                  placeholder="https://www.website.com"
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormDescription>URL of the bookmark</FormDescription>
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
          <form.Field
            name="title"
            validators={{
              onSubmit: z.string().min(1, 'Title cannot be empty'),
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Label>Title</Label>
                <div className="flex gap-2">
                  <Input
                    value={field.state.value}
                    onChange={(ev) => field.handleChange(ev.target.value)}
                  />
                </div>
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
          <DialogFooter>
            <Button
              disabled={scrapeBookmark.isPending || createBookmark.isPending}
            >
              Submit
            </Button>
          </DialogFooter>
        </div>
      </form>
    </DialogContent>
  )
}
