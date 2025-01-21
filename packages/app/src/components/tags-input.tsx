import { listTagsOptions } from '@colette/query'
import { Badge } from '@colette/react-ui/components/ui/badge'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@colette/react-ui/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@colette/react-ui/components/ui/popover'
import { cn } from '@colette/react-ui/lib/utils'
import type { FieldState, Updater } from '@tanstack/form-core'
import { useQuery } from '@tanstack/react-query'
import { Check, ChevronsUpDown, Plus } from 'lucide-react'
import { type FC, useState } from 'react'
import { useAPI } from '../lib/api-context'

export const TagsInput: FC<{
  state: FieldState<string[]>
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
            <span className="font-normal text-muted-foreground">
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
  state: FieldState<string[]>
  handleChange: (updater: Updater<string[]>) => void
}> = (props) => {
  const api = useAPI()

  const { data: tags } = useQuery(listTagsOptions({}, api))

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
              !tags?.data.find((tag) => tag.title === search) &&
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
            Create new tag "{search}"
          </CommandItem>
        </CommandGroup>
        <CommandGroup heading="Results">
          {tags?.data.map((tag) => (
            <CommandItem
              key={tag.title}
              className="justify-between"
              value={tag.title}
              onSelect={(value) =>
                props.handleChange((curr) =>
                  curr.includes(value)
                    ? curr.filter((tag) => tag !== value)
                    : [...curr, value].sort(),
                )
              }
            >
              {tag.title}
              {props.state.value.includes(tag.title) && <Check />}
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </Command>
  )
}
