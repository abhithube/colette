import { TagsInput } from '../../../components/tags-input'
import { useAPI } from '../../../lib/api-context'
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
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'
import { z } from 'zod'

export const EditFeedModal: FC<{
  feed: Feed
  close: () => void
}> = (props) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  const form = useForm({
    defaultValues: {
      title: props.feed.title ?? props.feed.originalTitle,
      tags: props.feed.tags?.map((tag) => tag.title) ?? [],
    },
    onSubmit: ({ value }) => {
      let title: string | null | undefined = value.title
      if (title === props.feed.title) {
        title = undefined
      } else if (title === props.feed.originalTitle) {
        if (!props.feed.title) {
          title = undefined
        } else {
          title = null
        }
      }

      let tags: string[] | undefined = value.tags
      if (props.feed.tags) {
        const current = props.feed.tags
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
        return props.close()
      }

      updateFeed({
        id: props.feed.id,
        body: {
          title,
          tags,
        },
      })
    },
  })

  const { mutateAsync: updateFeed, isPending } = useMutation(
    updateFeedOptions(api, queryClient, {
      onSuccess: () => {
        form.reset()
        props.close()
      },
    }),
  )

  useEffect(() => {
    form.reset()
  }, [form.reset, props.feed.id])

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
            Edit {props.feed.title ?? props.feed.originalTitle}
          </DialogTitle>
          <DialogDescription>Edit a feed&apos;s data.</DialogDescription>
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
