import { Field } from '@ark-ui/solid'
import { cva } from 'class-variance-authority'
import { type Component, splitProps } from 'solid-js'
import { cn } from '~/lib/utils'

const labelVariants = cva(
  'font-medium text-sm leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
)

export const Label: Component<Field.LabelProps> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return <Field.Label class={cn(labelVariants(), local.class)} {...others} />
}

export const FormMessage: Component<Field.ErrorTextProps> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <Field.ErrorText
      class={cn('font-medium text-destructive text-sm', local.class)}
      {...others}
    />
  )
}
