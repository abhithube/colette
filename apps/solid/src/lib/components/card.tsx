import {
  type Component,
  type ComponentProps,
  type ParentComponent,
  splitProps,
} from 'solid-js'
import { cn } from '../utils'

export const CardRoot: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      class={cn(
        'rounded-xl border bg-card text-card-foreground shadow',
        local.class,
      )}
      {...others}
    />
  )
}

export const CardHeader: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div class={cn('flex flex-col space-y-1.5 p-6', local.class)} {...others} />
  )
}

export const CardTitle: ParentComponent<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div
      class={cn('font-semibold leading-none tracking-tight', local.class)}
      {...others}
    />
  )
}

export const CardDescription: ParentComponent<ComponentProps<'div'>> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div class={cn('text-muted-foreground text-sm', local.class)} {...others} />
  )
}

export const CardContent: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return <div class={cn('p-6 pt-0', local.class)} {...others} />
}

export const CardFooter: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <div class={cn('flex items-center p-6 pt-0', local.class)} {...others} />
  )
}
