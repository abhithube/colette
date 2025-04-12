import { ark, HTMLArkProps } from '@ark-ui/react'

export type FaviconProps = HTMLArkProps<'img'>

export const Favicon = ({ src, alt, ...props }: FaviconProps) => {
  const domain = src ? new URL(src).hostname : undefined

  return (
    <ark.img
      className="size-4"
      src={
        domain ? `https://icons.duckduckgo.com/ip3/${domain}.ico` : undefined
      }
      {...props}
      alt={alt ?? domain}
    />
  )
}
