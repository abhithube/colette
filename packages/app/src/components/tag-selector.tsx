import type { Tag } from '@colette/core'
import {
  Combobox,
  HStack,
  Icon,
  IconButton,
  TagsInput,
  createListCollection,
} from '@colette/ui'
import type { FieldState, Updater } from '@tanstack/react-form'
import { Plus, X } from 'lucide-react'

type Props = {
  tags: Tag[]
  state: FieldState<string[]>
  handleChange: (updater: Updater<string[]>) => void
}

export function TagSelector({ tags, state, handleChange }: Props) {
  const collection = createListCollection({
    items: tags,
    itemToString: (item) => item.title,
    itemToValue: (item) => item.title,
  })

  return (
    <Combobox.Root
      asChild
      collection={collection}
      multiple
      allowCustomValue
      openOnClick
      value={state.value}
      onValueChange={(details) => handleChange(details.value.toSorted())}
    >
      <TagsInput.Root
        value={state.value}
        onValueChange={(details) => handleChange(details.value.toSorted())}
      >
        <Combobox.Label>Tags</Combobox.Label>
        <TagsInput.Context>
          {({ value, inputValue, clearInputValue }) => (
            <>
              <Combobox.Control asChild>
                <TagsInput.Control>
                  {value.map((item) => (
                    <TagsInput.Item key={item} index={item} value={item}>
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
              <Combobox.Positioner>
                <Combobox.Content>
                  <Combobox.ItemGroup>
                    <Combobox.ItemGroupLabel>Tags</Combobox.ItemGroupLabel>
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
  )
}
