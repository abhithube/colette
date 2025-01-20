import { createBookmarkOptions, scrapeBookmarkOptions } from '@colette/query'
import { FormDescription, FormMessage } from '@colette/react-ui/components/form'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@colette/react-ui/components/ui/dialog'
import { Input } from '@colette/react-ui/components/ui/input'
import { Label } from '@colette/react-ui/components/ui/label'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { z } from 'zod'
import { Route } from '../../bookmarks'

type Props = {
  close: () => void
}

export function AddBookmarkModal({ close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm({
    defaultValues: {
      url: '',
    },
    onSubmit: async ({ value }) => {
      const scraped = await scrapeBookmark(value)
      await createBookmark({
        url: scraped.link,
      })
    },
  })

  const navigate = useNavigate()

  const { mutateAsync: createBookmark, isPending: isPending1 } = useMutation(
    createBookmarkOptions(context.api, {
      onSuccess: async () => {
        form.reset()
        close()

        await navigate({
          to: '/bookmarks',
        })
      },
    }),
  )

  const { mutateAsync: scrapeBookmark, isPending: isPending2 } = useMutation(
    scrapeBookmarkOptions(context.api, {
      onSuccess: async () => {
        form.reset()
        close()

        await navigate({
          to: '/bookmarks',
        })
      },
    }),
  )

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
            {({ state, handleChange, handleBlur }) => (
              <div className="space-y-1">
                <Label>URL</Label>
                <Input
                  placeholder="https://www.website.com"
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <FormDescription>URL of the bookmark</FormDescription>
                <FormMessage>{state.meta.errors[0]?.toString()}</FormMessage>
              </div>
            )}
          </form.Field>
          <DialogFooter>
            <Button disabled={isPending1 || isPending2}>Submit</Button>
          </DialogFooter>
        </div>
      </form>
    </DialogContent>
  )
}
