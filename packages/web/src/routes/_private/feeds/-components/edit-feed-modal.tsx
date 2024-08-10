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
import { Input } from '@/components/ui/input'
import {
  MultiSelect,
  MultiSelectContent,
  MultiSelectItem,
  MultiSelectList,
  MultiSelectTrigger,
  MultiSelectValue,
} from '@/components/ui/multi-select'
import type { Feed } from '@colette/openapi'
import { listTagsOptions, updateFeedOptions } from '@colette/query'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { Route } from '../../../_private'

const formSchema = z.object({
  title: z.string().min(1),
  tags: z.string().array(),
})

type Props = {
  feed: Feed
  close: () => void
}

export function EditFeedModal({ feed, close }: Props) {
  const context = Route.useRouteContext()

  const { data: tags } = useQuery(
    listTagsOptions({}, context.profile.id, context.api),
  )

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })

  const { mutateAsync: updateFeed, isPending } = useMutation(
    updateFeedOptions(
      {
        onSuccess: async (data) => {
          await context.queryClient.setQueryData(['feeds', feed.id], data)
          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'feeds'],
          })

          close()
        },
      },
      context.api,
    ),
  )

  useEffect(() => {
    form.reset({
      tags: feed.tags?.map((tag) => tag.title) ?? [],
    })
  }, [form, feed])

  if (!tags) return

  return (
    <DialogContent>
      <Form {...form}>
        <form
          className="space-y-4"
          onSubmit={form.handleSubmit((data) =>
            updateFeed({
              id: feed.id,
              body: {
                title: data.title,
                tags: data.tags.map((title) => ({ title })),
              },
            }),
          )}
        >
          <DialogHeader>
            <DialogTitle>Edit {feed.title ?? feed.originalTitle}</DialogTitle>
            <DialogDescription>Edit a feed's data.</DialogDescription>
          </DialogHeader>
          <FormField
            control={form.control}
            name="title"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Title</FormLabel>
                <Input {...field} />
                <FormDescription>Custom title</FormDescription>
              </FormItem>
            )}
          />
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
                <FormDescription>Tags to add to the feed</FormDescription>
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
