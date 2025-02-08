import { TagsInput } from '../../../components/tags-input'
import type { Bookmark } from '@colette/core'
import { updateBookmarkOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'
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
  const api = useAPI()
  const queryClient = useQueryClient()

  const form = useForm({
    defaultValues: {
      tags: props.bookmark.tags?.map((tag) => tag.title) ?? [],
    },
    onSubmit: ({ value }) => {
      let tags: string[] | undefined = value.tags
      if (props.bookmark.tags) {
        const current = props.bookmark.tags
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

      if (tags === undefined) {
        return props.close()
      }

      updateBookmark({
        id: props.bookmark.id,
        body: {
          tags,
        },
      })
    },
  })

  const { mutateAsync: updateBookmark, isPending } = useMutation(
    updateBookmarkOptions(api, queryClient, {
      onSuccess: () => {
        form.reset()
        props.close()
      },
    }),
  )

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
            Edit {props.bookmark.title}
          </DialogTitle>
          <DialogDescription>Edit a feed&apos;s data.</DialogDescription>
        </DialogHeader>
        <div className="mt-4 flex flex-col items-stretch space-y-4">
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
