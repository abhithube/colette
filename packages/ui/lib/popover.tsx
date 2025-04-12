import { cn } from './utils'
import { Popover } from '@ark-ui/react'

export type RootProps = Popover.RootProps

export const Root = Popover.Root

export type TriggerProps = Popover.TriggerProps

export const Trigger = Popover.Trigger

export type AnchorProps = Popover.AnchorProps

export const Anchor = Popover.Anchor

export type ContentProps = Popover.ContentProps

export const Content = ({ className, ...props }: ContentProps) => {
  return (
    <Popover.Positioner>
      <Popover.Content
        className={cn(
          'bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-72 rounded-md border p-4 shadow-md outline-hidden',
          className,
        )}
        {...props}
      />
    </Popover.Positioner>
  )
}

export { Portal, type PortalProps } from '@ark-ui/react'
