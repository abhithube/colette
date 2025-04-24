import { cn } from './utils'
import { Steps, useStepsContext } from '@ark-ui/react'

export type ProviderProps = Steps.RootProviderProps

export const Provider = Steps.RootProvider

export type ListProps = Steps.ListProps

export const List = ({ className, ...props }: ListProps) => {
  return <Steps.List className={cn('flex gap-4', className)} {...props} />
}

export type ItemProps = Steps.ItemProps

export const Item = ({ className, index, ...props }: ItemProps) => {
  const context = useStepsContext()

  const extraProps: { 'data-last'?: string } = {}
  if (index === context.count - 1) {
    extraProps['data-last'] = ''
  }

  return (
    <Steps.Item
      className={cn(
        'flex flex-1 items-center gap-4 data-[last]:flex-0',
        className,
      )}
      index={index}
      {...extraProps}
      {...props}
    />
  )
}

export type TriggerProps = Steps.TriggerProps

export const Trigger = ({ className, ...props }: TriggerProps) => {
  return (
    <Steps.Trigger
      className={cn('flex flex-col items-center gap-1', className)}
      {...props}
    />
  )
}

export type IndicatorProps = Steps.IndicatorProps

export const Indicator = ({ className, ...props }: IndicatorProps) => {
  return (
    <Steps.Indicator
      className={cn(
        'data-[current]:border-primary data-[current]:text-primary data-[complete]:bg-primary data-[complete]:text-primary-foreground flex size-8 items-center justify-center rounded-full border',
        className,
      )}
      {...props}
    />
  )
}

export type SeparatorProps = Steps.SeparatorProps

export const Separator = ({ className, ...props }: SeparatorProps) => {
  return (
    <Steps.Separator
      className={cn(
        'bg-border data-[complete]:bg-primary -mx-1 h-px flex-1',
        className,
      )}
      {...props}
    />
  )
}

export type ContentProps = Steps.ContentProps

export const Content = Steps.Content

export type PrevTriggerProps = Steps.PrevTriggerProps

export const PrevTrigger = Steps.PrevTrigger

export type NextTriggerProps = Steps.NextTriggerProps

export const NextTrigger = Steps.NextTrigger

export { useSteps } from '@ark-ui/react'
