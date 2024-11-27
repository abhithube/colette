import type { Bookmark } from '@colette/core'
import { listTagsOptions, updateBookmarkOptions } from '@colette/query'
import { Button, Dialog, Flex, IconButton, VStack } from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQuery } from '@tanstack/react-query'
import { X } from 'lucide-react'
import { useEffect } from 'react'
import { TagSelector } from '../../../../components/tag-selector'
import { Route } from '../../../_private'

type Props = {
  bookmark: Bookmark
  close: () => void
}

export function EditBookmarkModal({ bookmark, close }: Props) {
  const context = Route.useRouteContext()

  const { data: tags } = useQuery(
    listTagsOptions({}, context.profile.id, context.api),
  )

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
    updateBookmarkOptions(
      {
        onSuccess: async (data) => {
          form.reset()
          close()

          await context.queryClient.setQueryData(['feeds', bookmark.id], data)
          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'feeds'],
          })
        },
      },
      context.api,
    ),
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    form.reset()
  }, [form.reset, bookmark.id])

  if (!tags) return

  return (
    <Dialog.Content maxW="md" p={6}>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <Dialog.Title lineClamp={1}>Edit {bookmark.title}</Dialog.Title>
        <Dialog.Description>Edit a feed's data.</Dialog.Description>
        <VStack alignItems="stretch" spaceY={4} mt={4}>
          <form.Field name="tags">
            {({ state, handleChange }) => (
              <TagSelector
                tags={tags.data}
                state={state}
                handleChange={handleChange}
              />
            )}
          </form.Field>
          <Flex justify="end">
            <Button loading={isPending}>Submit</Button>
          </Flex>
        </VStack>
      </form>
      <Dialog.CloseTrigger asChild position="absolute" top="2" right="2">
        <IconButton variant="ghost" size="sm">
          <X />
        </IconButton>
      </Dialog.CloseTrigger>
    </Dialog.Content>
  )
}
