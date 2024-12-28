import {
  type Component,
  type ComponentProps,
  type ParentComponent,
  splitProps,
} from 'solid-js'
import { cn } from '../utils'

export const Root: Component<ComponentProps<'div'>> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return (
    <div
      class={cn(
        'rounded-xl border bg-card text-card-foreground shadow',
        local.class,
      )}
      {...rest}
    />
  )
}

export const Header: Component<ComponentProps<'div'>> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return (
    <div class={cn('flex flex-col space-y-1.5 p-6', local.class)} {...rest} />
  )
}

export const Title: ParentComponent<ComponentProps<'div'>> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return (
    <div
      class={cn('font-semibold leading-none tracking-tight', local.class)}
      {...rest}
    />
  )
}

export const Description: ParentComponent<ComponentProps<'div'>> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return (
    <div class={cn('text-sm text-muted-foreground', local.class)} {...rest} />
  )
}

export const Content: Component<ComponentProps<'div'>> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return <div class={cn('p-6 pt-0', local.class)} {...rest} />
}

export const Footer: Component<ComponentProps<'div'>> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return <div class={cn('flex items-center p-6 pt-0', local.class)} {...rest} />
}
