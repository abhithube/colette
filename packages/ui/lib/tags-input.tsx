import { cn } from './utils'
import { TagsInput } from '@ark-ui/react'

export type RootProps = TagsInput.RootProps

export const Root = ({ children, ...props }: RootProps) => {
  return (
    <TagsInput.Root {...props}>
      {children}
      <TagsInput.HiddenInput />
    </TagsInput.Root>
  )
}

export type ContextProps = TagsInput.ContextProps

export const Context = TagsInput.Context

export type LabelProps = TagsInput.LabelProps

export const Label = TagsInput.Label

export type InputProps = TagsInput.InputProps

export const Input = ({ className, ...props }: InputProps) => {
  return (
    <TagsInput.Control className="flex h-9 items-center gap-2 border-b px-3">
      {/* <Search className="size-4 shrink-0 opacity-50" /> */}
      <TagsInput.Input
        className={cn(
          'placeholder:text-muted-foreground flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-hidden disabled:cursor-not-allowed disabled:opacity-50',
          className,
        )}
        {...props}
      />
    </TagsInput.Control>
  )
}
