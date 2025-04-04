import { listTagsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { type FieldState, type Updater } from '@tanstack/react-form'
import { useQuery } from '@tanstack/react-query'
import { Check, ChevronsUpDown, Plus } from 'lucide-react'
import { type FC, useState } from 'react'
import { Badge } from '~/components/ui/badge'
import { Button } from '~/components/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '~/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '~/components/ui/popover'
import { cn } from '~/lib/utils'

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

export const TagsInput: FC<{
  state: TagsState
  handleChange: (updater: Updater<string[]>) => void
}> = (props) => {
  const [isOpen, setOpen] = useState(false)

  return (
    <Popover open={isOpen} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
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
      </PopoverTrigger>
      <PopoverContent className="w-[--radix-popover-trigger-width] p-0">
        <TagsInner {...props} />
      </PopoverContent>
    </Popover>
  )
}

export const TagsInner: FC<{
  state: TagsState
  handleChange: (updater: Updater<string[]>) => void
}> = (props) => {
  const api = useAPI()

  const query = useQuery(listTagsOptions(api))

  const [search, setSearch] = useState('')

  return (
    <Command>
      <CommandInput
        value={search}
        placeholder="Search tags..."
        onValueChange={setSearch}
      />
      <CommandList>
        <CommandEmpty>No tags found.</CommandEmpty>
        <CommandGroup
          className={cn(
            'hidden',
            search !== '' &&
              !props.state.value.find((tag) => tag === search) &&
              !query.data?.data.find(
                (details) => details.tag.title === search,
              ) &&
              'block',
          )}
        >
          <CommandItem
            value={search}
            onSelect={(value) =>
              props.handleChange((curr) => [...curr, value].sort())
            }
          >
            <Plus />
            {`Create new tag "${search}"`}
          </CommandItem>
        </CommandGroup>
        <CommandGroup heading="Results">
          {query.data?.data.map((details) => (
            <CommandItem
              key={details.tag.id}
              className="justify-between"
              value={details.tag.id}
              onSelect={(value) =>
                props.handleChange((curr) =>
                  curr.includes(value)
                    ? curr.filter((tag) => tag !== value)
                    : [...curr, value].sort(),
                )
              }
            >
              {details.tag.title}
              {props.state.value.includes(details.tag.id) && <Check />}
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </Command>
  )
}
