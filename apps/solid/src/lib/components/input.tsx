import { Field } from '@ark-ui/solid'
import { type Component, splitProps } from 'solid-js'
import { cn } from '~/lib/utils'

export const Input: Component<Field.InputProps> = (props) => {
  const [local, others] = splitProps(props, ['class'])

  return (
    <Field.Input
      class={cn(
        'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-base ring-offset-background file:border-0 file:bg-transparent file:font-medium file:text-foreground file:text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
        local.class,
      )}
      {...others}
    />
  )
}
