import type { Bookmark } from '@colette/core'
import { useUpdateBookmarkMutation } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { type FC, useEffect } from 'react'
import { TagsInput } from '~/components/tags-input'
import { Button } from '~/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import { Label } from '~/components/ui/label'

export const EditBookmarkModal: FC<{
  bookmark: Bookmark
  close: () => void
}> = (props) => {
  const form = useForm({
    defaultValues: {
      tags: props.bookmark.tags?.map((tag) => tag.id) ?? [],
    },
    onSubmit: ({ value }) => {
      let tags: string[] | undefined = value.tags
      if (props.bookmark.tags) {
        const current = props.bookmark.tags
        if (
          tags?.length === current.length &&
          tags.every((id) => current.find((tag) => tag.id === id) !== undefined)
        ) {
          tags = undefined
        }
      } else if (tags.length === 0) {
        tags = undefined
      }

      if (tags === undefined) {
        return props.close()
      }

      updateBookmark.mutate(
        {
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

  const updateBookmark = useUpdateBookmarkMutation(props.bookmark.id)

  useEffect(() => {
    form.reset()
  }, [form, props.bookmark.id])

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
            Edit <span className="text-primary">{props.bookmark.title}</span>
          </DialogTitle>
          <DialogDescription>{"Edit a feed's data."}</DialogDescription>
        </DialogHeader>
        <div className="mt-4 flex flex-col items-stretch space-y-4">
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
            <Button disabled={updateBookmark.isPending}>Submit</Button>
          </DialogFooter>
        </div>
      </form>
    </DialogContent>
  )
}
