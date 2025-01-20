import type { Feed } from '@colette/core'
import { updateFeedOptions } from '@colette/query'
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
import { useEffect } from 'react'
import { z } from 'zod'
import { TagsInput } from '../../../../components/tags-input'
import { Route } from '../../../_private'

type Props = {
  feed: Feed
  close: () => void
}

export function EditFeedModal({ feed, close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm({
    defaultValues: {
      title: feed.title ?? feed.originalTitle,
      tags: feed.tags?.map((tag) => tag.title) ?? [],
    },
    onSubmit: ({ value }) => {
      let title: string | null | undefined = value.title
      if (title === feed.title) {
        title = undefined
      } else if (title === feed.originalTitle) {
        if (!feed.title) {
          title = undefined
        } else {
          title = null
        }
      }

      let tags: string[] | undefined = value.tags
      if (feed.tags) {
        const current = feed.tags
        if (
          tags?.length === current.length &&
          tags.every(
            (title) => current.find((tag) => tag.title === title) !== undefined,
          )
        ) {
          tags = undefined
        }
      } else if (tags.length === 0) {
        tags = undefined
      }

      if (title === undefined && tags === undefined) {
        return close()
      }

      updateFeed({
        id: feed.id,
        body: {
          title,
          tags,
        },
      })
    },
  })

  const { mutateAsync: updateFeed, isPending } = useMutation(
    updateFeedOptions(context.api, {
      onSuccess: async (data) => {
        form.reset()
        close()

        await context.queryClient.setQueryData(['feeds', feed.id], data)
        await context.queryClient.invalidateQueries({
          queryKey: ['feeds'],
        })
      },
    }),
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    form.reset()
  }, [form.reset, feed.id])

  return (
    <DialogContent className="max-w-md p-6">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <DialogHeader>
          <DialogTitle className="line-clamp-1">
            Edit {feed.title ?? feed.originalTitle}
          </DialogTitle>
          <DialogDescription>Edit a feed's data.</DialogDescription>
        </DialogHeader>
        <div className="mt-4 flex flex-col items-stretch space-y-4">
          <form.Field
            name="title"
            validators={{
              onBlur: z.string().min(1, "Title can't be empty"),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <div className="space-y-1">
                <Label>Title</Label>
                <Input
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <FormDescription>Custom title</FormDescription>
                <FormMessage>{state.meta.errors[0]?.toString()}</FormMessage>
              </div>
            )}
          </form.Field>
          <form.Field name="tags">
            {({ state, handleChange }) => (
              <div className="space-y-1">
                <Label>Tags</Label>
                <TagsInput state={state} handleChange={handleChange} />
              </div>
            )}
          </form.Field>
          <DialogFooter>
            <Button disabled={isPending}>Submit</Button>
          </DialogFooter>
        </div>
      </form>
    </DialogContent>
  )
}
