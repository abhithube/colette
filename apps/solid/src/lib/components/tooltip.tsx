import { Tooltip as TooltipPrimitive } from '@ark-ui/solid'
import { type Component, splitProps } from 'solid-js'
import { cn } from '../utils'

export const Tooltip: Component<TooltipPrimitive.RootProps> = (props) => {
  const [local, others] = splitProps(props, ['positioning'])

  return <TooltipPrimitive.Root positioning={local.positioning} {...others} />
}

export const TooltipTrigger: Component<TooltipPrimitive.TriggerProps> = (
  props,
) => {
  return <TooltipPrimitive.Trigger {...props} />
}

export const TooltipContent: Component<TooltipPrimitive.ContentProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <TooltipPrimitive.Content
      class={cn(
        'fade-in-0 zoom-in-95 data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 animate-in overflow-hidden rounded-md border bg-popover px-3 py-1.5 text-popover-foreground text-sm shadow-md data-[state=closed]:animate-out',
        local.class,
      )}
      {...others}
    />
  )
}
