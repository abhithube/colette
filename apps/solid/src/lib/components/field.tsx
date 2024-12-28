import { Field } from '@ark-ui/solid'
import { type Component, splitProps } from 'solid-js'
import { cn } from '../utils'
import { cva } from 'class-variance-authority'

export const labelVariants = cva(
  'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
)

export const Root: Component<Field.RootProps> = (props) => {
  return <Field.Root {...props} />
}

export const Input: Component<Field.InputProps> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return (
    <Field.Input
      class={cn(
        'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-base ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
        local.class,
      )}
      {...rest}
    />
  )
}

export const Label: Component<Field.LabelProps> = (props) => {
  const [local, rest] = splitProps(props, ['class'])

  return <Field.Label class={cn(labelVariants(), local.class)} {...rest} />
}
