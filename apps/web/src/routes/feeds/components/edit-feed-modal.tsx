import { TagsInput } from '../../../components/tags-input'
import type { Feed } from '@colette/core'
import { updateFeedOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'
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

export const EditFeedModal: FC<{
  feed: Feed
  close: () => void
}> = (props) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  const form = useForm({
    defaultValues: {
      title: props.feed.title,
      tags: props.feed.tags?.map((tag) => tag.title) ?? [],
    },
    onSubmit: ({ value }) => {
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

      if (value.title === props.feed.title && tags === undefined) {
        return props.close()
      }

      updateFeed({
        id: props.feed.id,
        body: {
          title: value.title,
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
  }, [form, props.feed.id])

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
            Edit {props.feed.title}
          </DialogTitle>
          <DialogDescription>{"Edit a feed's data."}</DialogDescription>
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
