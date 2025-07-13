import { listTagsOptions } from '@colette/query'
import { Badge, Button, Combobox, Popover } from '@colette/ui'
import { cn, createListCollection } from '@colette/ui/utils'
import type { FieldState, Updater } from '@tanstack/react-form'
import { useQuery } from '@tanstack/react-query'
import { Check, ChevronsUpDown, Plus } from 'lucide-react'
import { useMemo, useState } from 'react'

type TagsState = FieldState<
  any,
  any,
  string[],
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any
>

export const TagsInput = (props: {
  state: TagsState
  handleChange: (updater: Updater<string[]>) => void
}) => {
  const [isOpen, setOpen] = useState(false)

  return (
    <Popover.Root
      open={isOpen}
      onOpenChange={(details) => setOpen(details.open)}
      positioning={{ sameWidth: true }}
    >
      <Popover.Trigger asChild>
        <Button
          className="flex h-full w-full justify-between"
          variant="outline"
        >
          {props.state.value.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {props.state.value.map((tag) => (
                <Badge key={tag} className="rounded-sm">
                  {tag}
                </Badge>
              ))}
            </div>
          ) : (
            <span className="text-muted-foreground font-normal">
              Select tags...
            </span>
          )}
          <ChevronsUpDown className="text-muted-foreground" />
        </Button>
      </Popover.Trigger>
      <Popover.Content>
        <TagsInner {...props} />
      </Popover.Content>
    </Popover.Root>
  )
}

export const TagsInner = (props: {
  state: TagsState
  handleChange: (updater: Updater<string[]>) => void
}) => {
  const query = useQuery(listTagsOptions())

  const collection = useMemo(
    () => createListCollection({ items: props.state.value }),
    [props.state.value],
  )

  const [search, setSearch] = useState('')

  return (
    <Combobox.Root
      collection={collection}
      onInputValueChange={(details) => setSearch(details.inputValue)}
      onValueChange={(details) => props.handleChange(details.value.toSorted())}
    >
      <Combobox.Input placeholder="Search tags..." />
      <Combobox.List>
        <Combobox.ItemGroup
          className={cn(
            'hidden',
            search !== '' &&
              !props.state.value.find((tag) => tag === search) &&
              !query.data?.items.find(
                (details) => details.tag.title === search,
              ) &&
              'block',
          )}
        >
          <Combobox.Item item={search}>
            <Plus />
            {`Create new tag "${search}"`}
          </Combobox.Item>
        </Combobox.ItemGroup>
        <Combobox.ItemGroup>
          <Combobox.ItemGroupLabel>Results</Combobox.ItemGroupLabel>
          {query.data?.items.map((details) => (
            <Combobox.Item
              key={details.tag.id}
              className="justify-between"
              item={details.tag.id}
            >
              {details.tag.title}
              {props.state.value.includes(details.tag.id) && <Check />}
            </Combobox.Item>
          ))}
        </Combobox.ItemGroup>
      </Combobox.List>
    </Combobox.Root>
  )
}
