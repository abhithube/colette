import { cn } from './utils'
import { Tooltip } from '@ark-ui/react'

export type RootProps = Tooltip.RootProps

export const Root = Tooltip.Root

export type TriggerProps = Tooltip.TriggerProps

export const Trigger = Tooltip.Trigger

export type ContentProps = Tooltip.ContentProps

export const Content = ({ className, children, ...props }: ContentProps) => {
  return (
    <Tooltip.Positioner>
      <Tooltip.Content
        className={cn(
          'bg-primary text-primary-foreground animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-fit rounded-md px-3 py-1.5 text-xs text-balance',
          className,
        )}
        {...props}
      >
        {children}
      </Tooltip.Content>
    </Tooltip.Positioner>
  )
}
