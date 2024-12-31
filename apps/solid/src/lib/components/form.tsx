import { Field } from '@ark-ui/solid'
import { type Component, splitProps } from 'solid-js'
import { cn } from '~/lib/utils'

export const FormControl: Component<Field.RootProps> = (props) => {
  return <Field.Root {...props} />
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
