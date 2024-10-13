import type { Feed } from '@colette/core'
import { listTagsOptions, updateFeedOptions } from '@colette/query'
import {
  Button,
  Combobox,
  Dialog,
  Field,
  Fieldset,
  Flex,
  HStack,
  Icon,
  IconButton,
  Switch,
  TagsInput,
  VStack,
  createListCollection,
} from '@colette/ui'
import { useForm } from '@tanstack/react-form'
import { useMutation, useQuery } from '@tanstack/react-query'
import { zodValidator } from '@tanstack/zod-form-adapter'
import { Plus, X } from 'lucide-react'
import { useEffect } from 'react'
import { z } from 'zod'
import { Route } from '../../../_private'

type Props = {
  feed: Feed
  close: () => void
}

export function EditFeedModal({ feed, close }: Props) {
  const context = Route.useRouteContext()

  const { data: tags } = useQuery(
    listTagsOptions({}, context.profile.id, context.api),
  )

  const form = useForm({
    defaultValues: {
      title: feed.title ?? feed.originalTitle,
      pinned: feed.pinned,
      tags: feed.tags?.map((tag) => tag.title) ?? [],
    },
    onSubmit: ({ value }) => {
      let title: string | null | undefined = value.title
      if (title === feed.title) {
        title = undefined
      } else if (title === feed.originalTitle) {
        if (!feed.title) {
          title = undefined
        } else {
          title = null
        }
      }

      const pinned = value.pinned === feed.pinned ? undefined : value.pinned

      let tags: string[] | undefined = value.tags
      if (feed.tags) {
        const current = feed.tags
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

      if (title === undefined && pinned === undefined && tags === undefined) {
        return close()
      }

      updateFeed({
        id: feed.id,
        body: {
          title,
          pinned,
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

  const { mutateAsync: updateFeed, isPending } = useMutation(
    updateFeedOptions(
      {
        onSuccess: async (data) => {
          form.reset()
          close()

          await context.queryClient.setQueryData(['feeds', feed.id], data)
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
  }, [form.reset, feed.id])

  if (!tags) return

  const collection = createListCollection({
    items: tags.data,
    itemToString: (item) => item.title,
    itemToValue: (item) => item.title,
  })

  return (
    <Dialog.Content maxW="md" p={6}>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          form.handleSubmit()
        }}
      >
        <Dialog.Title lineClamp={1}>
          Edit {feed.title ?? feed.originalTitle}
        </Dialog.Title>
        <Dialog.Description>Edit a feed's data.</Dialog.Description>
        <VStack alignItems="stretch" spaceY={4} mt={4}>
          <form.Field
            name="title"
            validatorAdapter={zodValidator()}
            validators={{
              onBlur: z.string().min(1, "Title can't be empty"),
            }}
          >
            {({ state, handleChange, handleBlur }) => (
              <Field.Root invalid={state.meta.errors.length > 0}>
                <Field.Label>Title</Field.Label>
                <Field.Input
                  value={state.value}
                  onChange={(e) => handleChange(e.target.value)}
                  onBlur={handleBlur}
                />
                <Field.HelperText>Custom title</Field.HelperText>
                <Field.ErrorText>
                  {state.meta.errors[0]?.toString()}
                </Field.ErrorText>
              </Field.Root>
            )}
          </form.Field>
          <form.Field name="pinned">
            {({ state, handleChange }) => (
              <Fieldset.Root paddingBlock={0} borderTop="none">
                <Fieldset.Legend>Pinned</Fieldset.Legend>
                <Fieldset.HelperText>
                  Should the feed be pinned to the sidebar?
                </Fieldset.HelperText>
                <Field.Root>
                  <Switch
                    checked={state.value}
                    onCheckedChange={(details) => handleChange(details.checked)}
                  />
                </Field.Root>
              </Fieldset.Root>
            )}
          </form.Field>
          <form.Field name="tags">
            {({ state, handleChange }) => (
              <Combobox.Root
                asChild
                collection={collection}
                multiple
                allowCustomValue
                openOnClick
                value={state.value}
                onValueChange={(details) =>
                  handleChange(details.value.toSorted())
                }
              >
                <TagsInput.Root
                  value={state.value}
                  onValueChange={(details) =>
                    handleChange(details.value.toSorted())
                  }
                >
                  <Combobox.Label>Tags</Combobox.Label>
                  <TagsInput.Context>
                    {({ value, inputValue, clearInputValue }) => (
                      <>
                        <Combobox.Control asChild>
                          <TagsInput.Control>
                            {value.map((item) => (
                              <TagsInput.Item
                                key={item}
                                index={item}
                                value={item}
                              >
                                <TagsInput.ItemPreview>
                                  <TagsInput.ItemText>
                                    {item}
                                  </TagsInput.ItemText>
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
                        <Combobox.Positioner>
                          <Combobox.Content>
                            <Combobox.ItemGroup>
                              <Combobox.ItemGroupLabel>
                                Tags
                              </Combobox.ItemGroupLabel>
                              {collection.items
                                .filter((tag) => !value.includes(tag.title))
                                .filter((tag) =>
                                  inputValue.length > 0
                                    ? tag.title
                                        .toLowerCase()
                                        .includes(inputValue.toLowerCase())
                                    : true,
                                )
                                .map((tag) => (
                                  <Combobox.Item key={tag.title} item={tag}>
                                    {tag.title}
                                  </Combobox.Item>
                                ))}
                              {inputValue.length > 0 &&
                                !collection.has(inputValue) &&
                                !value.includes(inputValue) && (
                                  <Combobox.Item
                                    key={inputValue}
                                    item={{ id: '', title: inputValue }}
                                    onClick={clearInputValue}
                                  >
                                    <HStack>
                                      <Icon>
                                        <Plus />
                                      </Icon>
                                      Create new tag
                                    </HStack>
                                  </Combobox.Item>
                                )}
                            </Combobox.ItemGroup>
                          </Combobox.Content>
                        </Combobox.Positioner>
                      </>
                    )}
                  </TagsInput.Context>
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
