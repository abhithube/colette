import {
  Button,
  Combobox,
  Dialog,
  Flex,
  IconButton,
  TagsInput,
  VStack,
} from '@colette/components'
import type { Bookmark, Tag } from '@colette/core'
import { listTagsOptions, updateBookmarkOptions } from '@colette/query'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQuery } from '@tanstack/react-query'
import { X } from 'lucide-react'
import { useEffect } from 'react'
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
          tags: tags
            ? {
                data: tags,
                action: 'set',
              }
            : undefined,
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
              <Combobox.Root
                asChild
                items={tags.data}
                itemToString={(item) => (item as Tag).title}
                itemToValue={(item) => (item as Tag).title}
                multiple
                openOnClick
                closeOnSelect
                onValueChange={(details) => handleChange(details.value)}
              >
                <TagsInput.Root
                  value={state.value ?? []}
                  onValueChange={(details) => handleChange(details.value)}
                >
                  <Combobox.Label>Tags</Combobox.Label>
                  <TagsInput.Context>
                    {({ value }) => (
                      <Combobox.Control asChild>
                        <TagsInput.Control>
                          {value.map((item) => (
                            <TagsInput.Item
                              key={item}
                              index={item}
                              value={item}
                            >
                              <TagsInput.ItemPreview>
                                <TagsInput.ItemText>{item}</TagsInput.ItemText>
                                <TagsInput.ItemDeleteTrigger asChild>
                                  <IconButton variant="link" size="xs">
                                    <X />
                                  </IconButton>
                                </TagsInput.ItemDeleteTrigger>
                              </TagsInput.ItemPreview>
                              <TagsInput.ItemInput />
                              <TagsInput.HiddenInput />
                            </TagsInput.Item>
                          ))}
                          <Combobox.Input placeholder="Add tag..." asChild>
                            <TagsInput.Input />
                          </Combobox.Input>
                        </TagsInput.Control>
                      </Combobox.Control>
                    )}
                  </TagsInput.Context>
                  <Combobox.Positioner>
                    <Combobox.Content>
                      <Combobox.ItemGroup>
                        <Combobox.ItemGroupLabel>Tags</Combobox.ItemGroupLabel>
                        {tags.data
                          .filter((tag) => !state.value?.includes(tag.title))
                          .map((tag) => (
                            <Combobox.Item key={tag.id} item={tag}>
                              {tag.title}
                            </Combobox.Item>
                          ))}
                      </Combobox.ItemGroup>
                    </Combobox.Content>
                  </Combobox.Positioner>
                </TagsInput.Root>
              </Combobox.Root>
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
