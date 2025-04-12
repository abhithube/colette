import { cn } from './utils'
import { Combobox, HTMLArkProps, ark } from '@ark-ui/react'
import { Search } from 'lucide-react'

export type RootProps<T extends Combobox.CollectionItem> = Combobox.RootProps<T>

export const Root = <T extends Combobox.CollectionItem>({
  className,
  ...props
}: RootProps<T>) => (
  <Combobox.Root
    className={cn(
      'bg-popover text-popover-foreground flex size-full flex-col overflow-hidden rounded-md',
      className,
    )}
    {...props}
  />
)

export type InputProps = Combobox.InputProps

export const Input = ({ className, ...props }: InputProps) => {
  return (
    <Combobox.Control className="flex h-9 items-center gap-2 border-b px-3">
      <Search className="size-4 shrink-0 opacity-50" />
      <Combobox.Input
        className={cn(
          'placeholder:text-muted-foreground flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-hidden disabled:cursor-not-allowed disabled:opacity-50',
          className,
        )}
        {...props}
      />
    </Combobox.Control>
  )
}

export type ListProps = Combobox.ListProps

export const List = ({ className, ...props }: ListProps) => {
  return (
    <Combobox.List
      className={cn(
        'max-h-[300px] scroll-py-1 overflow-x-hidden overflow-y-auto',
        className,
      )}
      {...props}
    />
  )
}

export type ItemGroupProps = Combobox.ItemGroupProps

export const ItemGroup = ({ className, ...props }: ItemGroupProps) => (
  <Combobox.ItemGroup
    className={cn('text-foreground overflow-hidden p-1', className)}
    {...props}
  />
)

export type ItemGroupLabelProps = Combobox.ItemGroupLabelProps

export const ItemGroupLabel = ({
  className,
  ...props
}: ItemGroupLabelProps) => (
  <Combobox.ItemGroupLabel
    className={cn(
      'text-muted-foreground px-2 py-1.5 text-xs font-medium',
      className,
    )}
    {...props}
  />
)

export type SeparatorProps = HTMLArkProps<'div'>

export const Separator = ({ className, ...props }: SeparatorProps) => (
  <ark.div
    role="separator"
    data-scope="combobox"
    data-part="separator"
    className={cn('bg-border -mx-1 h-px', className)}
    {...props}
  />
)

export type ItemProps = Combobox.ItemProps

export const Item = ({ className, ...props }: ItemProps) => (
  <Combobox.Item
    className={cn(
      "data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
      className,
    )}
    {...props}
  />
)

export type ShortcutProps = HTMLArkProps<'span'>

export const Shortcut = ({ className, ...props }: ShortcutProps) => {
  return (
    <ark.span
      data-scope="combobox"
      data-part="shortcut"
      className={cn(
        'text-muted-foreground ml-auto text-xs tracking-widest',
        className,
      )}
      {...props}
    />
  )
}
