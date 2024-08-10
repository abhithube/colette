import { Icon } from '@/components/icon'
import { Button } from '@/components/ui/button'
import {
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Form,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
} from '@/components/ui/form'
import {
  MultiSelect,
  MultiSelectContent,
  MultiSelectItem,
  MultiSelectList,
  MultiSelectTrigger,
  MultiSelectValue,
} from '@/components/ui/multi-select'
import type { Bookmark } from '@colette/openapi'
import { listTagsOptions, updateBookmarkOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../../../_private'

const formSchema = z.object({
  tags: z.string().array(),
})

type Props = {
  bookmark: Bookmark
  close: () => void
}

export function EditBookmarkModal({ bookmark, close }: Props) {
  const context = Route.useRouteContext()

  const { data: tags } = useQuery(
    listTagsOptions({}, context.profile.id, context.api),
  )

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })

  const { mutateAsync: updateBookmark, isPending } = useMutation(
    updateBookmarkOptions(
      {
        onSuccess: close,
      },
      context.api,
    ),
  )

  useEffect(() => {
    form.reset({
      tags: bookmark.tags?.map((tag) => tag.title) ?? [],
    })
  }, [form, bookmark])

  if (!tags) return

  return (
    <DialogContent>
      <Form {...form}>
        <form
          className="space-y-4"
          onSubmit={form.handleSubmit((data) => {
            let tags: string[] | undefined = data.tags
            if (bookmark.tags) {
              const current = bookmark.tags
              if (
                tags.length === current.length &&
                tags.every(
                  (title) =>
                    current.find((tag) => tag.title === title) !== undefined,
                )
              ) {
                tags = undefined
              }
            }

            if (tags === undefined) {
              return close()
            }

            updateBookmark({
              id: bookmark.id,
              body: {
                tags: tags.map((title) => ({ title })),
              },
            })
          })}
        >
          <DialogHeader>
            <DialogTitle>Edit {bookmark.title}</DialogTitle>
            <DialogDescription>Edit a bookmark's data.</DialogDescription>
          </DialogHeader>
          <FormField
            control={form.control}
            name="tags"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Tags</FormLabel>
                <MultiSelect
                  onValueChange={(value) => {
                    form.setValue('tags', value)
                  }}
                  {...field}
                >
                  <MultiSelectTrigger>
                    <MultiSelectValue placeholder="Select tags..." />
                  </MultiSelectTrigger>
                  <MultiSelectContent>
                    <MultiSelectList>
                      {tags.data.map((tag) => (
                        <MultiSelectItem key={tag.id} value={tag.title}>
                          {tag.title}
                        </MultiSelectItem>
                      ))}
                    </MultiSelectList>
                  </MultiSelectContent>
                </MultiSelect>
                <FormDescription>Tags to add to the bookmark</FormDescription>
              </FormItem>
            )}
          />
          <DialogFooter>
            <Button disabled={isPending}>
              {isPending && (
                <Icon className="mr-2 animate-spin" value={Loader2} />
              )}
              Submit
            </Button>
          </DialogFooter>
        </form>
      </Form>
    </DialogContent>
  )
}
