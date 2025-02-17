import { TagsInput } from '../../../components/tags-input'
import type { Feed } from '@colette/core'
import { useUpdateFeedMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
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

      updateFeed.mutate(
        {
          title: value.title,
          tags,
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

  const updateFeed = useUpdateFeedMutation(props.feed.id)

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
            {(field) => (
              <div className="space-y-1">
                <Label>Title</Label>
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                />
                <FormDescription>Custom title</FormDescription>
                <FormMessage>
                  {field.state.meta.errors[0]?.toString()}
                </FormMessage>
              </div>
            )}
          </form.Field>
          <form.Field name="tags">
            {(field) => (
              <div className="space-y-1">
                <Label>Tags</Label>
                <TagsInput
                  state={field.state}
                  handleChange={field.handleChange}
                />
              </div>
            )}
          </form.Field>
          <DialogFooter>
            <Button disabled={updateFeed.isPending}>Submit</Button>
          </DialogFooter>
        </div>
      </form>
    </DialogContent>
  )
}
