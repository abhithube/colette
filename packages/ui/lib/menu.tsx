import { cn } from './utils'
import { ark, type HTMLArkProps, Menu } from '@ark-ui/react'
import { Check, Circle } from 'lucide-react'

export type RootProps = Menu.RootProps

export const Root = Menu.Root

export type TriggerProps = Menu.TriggerProps

export const Trigger = Menu.Trigger

export type ItemGroupProps = Menu.ItemGroupProps

export const ItemGroup = Menu.ItemGroup

export type RadioItemGroupProps = Menu.RadioItemGroupProps

export const RadioItemGroup = Menu.RadioItemGroup

export type ContentProps = Menu.ContentProps

export const Content = ({ className, ...props }: ContentProps) => (
  <Menu.Positioner>
    <Menu.Content
      className={cn(
        'bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 min-w-[8rem] overflow-x-hidden overflow-y-auto rounded-md border p-1 shadow-md',
        className,
      )}
      {...props}
    />
  </Menu.Positioner>
)

export type ItemProps = Menu.ItemProps & {
  inset?: boolean
  variant?: 'default' | 'destructive'
}

export const Item = ({ className, inset, variant, ...props }: ItemProps) => (
  <Menu.Item
    data-inset={inset}
    data-variant={variant}
    className={cn(
      "data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[variant=destructive]:text-destructive data-[variant=destructive]:data-[highlighted]:bg-destructive/10 dark:data-[variant=destructive]:data-[highlighted]:bg-destructive/20 data-[variant=destructive]:data-[highlighted]:text-destructive data-[variant=destructive]:*:[svg]:!text-destructive [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 data-[inset]:pl-8 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
      className,
    )}
    {...props}
  />
)

export type CheckboxItemProps = Menu.CheckboxItemProps

export const CheckboxItem = ({
  className,
  children,
  checked,
  ...props
}: CheckboxItemProps) => {
  return (
    <Menu.CheckboxItem
      className={cn(
        "data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm py-1.5 pr-2 pl-8 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        className,
      )}
      checked={checked}
      {...props}
    >
      <span className="pointer-events-none absolute left-2 flex size-3.5 items-center justify-center">
        <Menu.ItemIndicator>
          <Check className="size-4" />
        </Menu.ItemIndicator>
      </span>
      {children}
    </Menu.CheckboxItem>
  )
}

export type RadioItemProps = Menu.RadioItemProps

export const RadioItem = ({
  className,
  children,
  ...props
}: RadioItemProps) => {
  return (
    <Menu.RadioItem
      className={cn(
        "data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm py-1.5 pr-2 pl-8 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        className,
      )}
      {...props}
    >
      <span className="pointer-events-none absolute left-2 flex size-3.5 items-center justify-center">
        <Menu.ItemIndicator>
          <Circle className="size-2 fill-current" />
        </Menu.ItemIndicator>
      </span>
      {children}
    </Menu.RadioItem>
  )
}

export type ItemGroupLabelProps = Menu.ItemGroupLabelProps & {
  inset?: boolean
}

export const ItemGroupLabel = ({
  className,
  inset,
  ...props
}: ItemGroupLabelProps) => (
  <Menu.ItemGroupLabel
    data-inset={inset}
    className={cn(
      'px-2 py-1.5 text-sm font-medium data-[inset]:pl-8',
      className,
    )}
    {...props}
  />
)

export type SeparatorProps = Menu.SeparatorProps

export const Separator = ({ className, ...props }: SeparatorProps) => {
  return (
    <Menu.Separator
      className={cn('bg-border -mx-1 my-1 h-px', className)}
      {...props}
    />
  )
}

export type ShortcutProps = HTMLArkProps<'span'>

export const Shortcut = ({ className, ...props }: ShortcutProps) => {
  return (
    <ark.span
      data-scope="menu"
      data-part="shortcut"
      className={cn(
        'text-muted-foreground ml-auto text-xs tracking-widest',
        className,
      )}
      {...props}
    />
  )
}

export { Portal, type PortalProps } from '@ark-ui/react'
