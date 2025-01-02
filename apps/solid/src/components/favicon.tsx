import { type Component, type ComponentProps, splitProps } from 'solid-js'
import { cn } from '~/lib/utils'

export const Favicon: Component<ComponentProps<'img'> & { url: string }> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class', 'url'])

  const domain = new URL(local.url).hostname

  return (
    <img
      class={cn('size-4', local.class)}
      src={`https://icons.duckduckgo.com/ip3/${domain}.ico`}
      width={16}
      height={16}
      {...others}
      alt={domain}
    />
  )
}
