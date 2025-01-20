import type { Bookmark } from '@colette/core'
import { updateBookmarkOptions } from '@colette/query'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@colette/react-ui/components/ui/dialog'
import { Label } from '@colette/react-ui/components/ui/label'
import { useForm } from '@tanstack/react-form'
import { useMutation } from '@tanstack/react-query'
import { useEffect } from 'react'
import { TagsInput } from '../../../../components/tags-input'
import { Route } from '../../../_private'

type Props = {
  bookmark: Bookmark
  close: () => void
}

export function EditBookmarkModal({ bookmark, close }: Props) {
  const context = Route.useRouteContext()

  const form = useForm({
    defaultValues: {
      tags: bookmark.tags?.map((tag) => tag.title) ?? [],
    },
    onSubmit: ({ value }) => {
      let tags: string[] | undefined = value.tags
      if (bookmark.tags) {
        const current = bookmark.tags
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
        return close()
      }

      updateBookmark({
        id: bookmark.id,
        body: {
          tags,
        },
      })
    },
  })

  const { mutateAsync: updateBookmark, isPending } = useMutation(
    updateBookmarkOptions(context.api, {
      onSuccess: async (data) => {
        form.reset()
        close()

        await context.queryClient.setQueryData(['bookmarks', bookmark.id], data)
        await context.queryClient.invalidateQueries({
          queryKey: ['bookmarks'],
        })
      },
    }),
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    form.reset()
  }, [form.reset, bookmark.id])

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
            Edit {bookmark.title}
          </DialogTitle>
          <DialogDescription>Edit a feed's data.</DialogDescription>
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
