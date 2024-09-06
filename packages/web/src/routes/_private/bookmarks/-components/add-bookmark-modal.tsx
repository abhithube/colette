import { Icon } from '@/components/icon'
import { Button } from '@/components/ui/button'
import {
  Form,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Dialog, IconButton } from '@colette/components'
import { BookmarkCreate } from '@colette/core'
import { createBookmarkOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2, X } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { Route } from '../../bookmarks'

type Props = {
  close: () => void
}

export function AddBookmarkModal({ close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm<BookmarkCreate>({
    resolver: zodResolver(BookmarkCreate),
    defaultValues: {
      url: '',
    },
  })

  const navigate = useNavigate()

  const { mutateAsync: createBookmark, isPending } = useMutation(
    createBookmarkOptions(
      {
        onSuccess: () => {
          form.reset()
          close()

          navigate({
            to: '/bookmarks/stash',
          })
        },
      },
      context.api,
    ),
  )

  return (
    <Dialog.Content>
      <Form {...form}>
        <form
          className="space-y-4"
          onSubmit={form.handleSubmit((data) => createBookmark(data))}
        >
          <Dialog.Title>Add Bookmark</Dialog.Title>
          <Dialog.Description>Add a bookmark to the stash.</Dialog.Description>
          <FormField
            control={form.control}
            name="url"
            render={({ field }) => (
              <FormItem>
                <FormLabel>URL</FormLabel>
                <Input {...field} />
                <FormDescription>URL of the bookmark</FormDescription>
              </FormItem>
            )}
          />
          <Button disabled={isPending}>
            {isPending && (
              <Icon className="mr-2 animate-spin" value={Loader2} />
            )}
            Submit
          </Button>
        </form>
      </Form>
      <Dialog.CloseTrigger asChild position="absolute" top="2" right="2">
        <IconButton aria-label="Close Dialog" variant="ghost" size="sm">
          <X />
        </IconButton>
      </Dialog.CloseTrigger>
    </Dialog.Content>
  )
}
