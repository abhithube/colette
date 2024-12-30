import { type HTMLArkProps, ark } from '@ark-ui/solid'
import { type Component, mergeProps, splitProps } from 'solid-js'

export type SeparatorProps = HTMLArkProps<'hr'> & {
  orientation?: 'horizontal' | 'vertical'
}

export const Separator: Component<SeparatorProps> = (props) => {
  const mergedProps = mergeProps(
    { orientation: 'horizontal' } as SeparatorProps,
    props,
  )

  const [local, others] = splitProps(mergedProps, ['orientation'])

  return (
    <ark.hr
      aria-orientation={local.orientation}
      data-orientation={local.orientation}
      {...others}
    />
  )
}
