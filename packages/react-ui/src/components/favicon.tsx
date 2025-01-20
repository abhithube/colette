import type { ComponentPropsWithRef, FC } from 'react'

const Favicon: FC<ComponentPropsWithRef<'img'> & { url: string }> = ({
  url,
  alt,
  ...props
}) => {
  const domain = new URL(url).hostname

  return (
    <img
      className="size-4"
      src={`https://icons.duckduckgo.com/ip3/${domain}.ico`}
      {...props}
      alt={alt ?? domain}
    />
  )
}
Favicon.displayName = 'Favicon'

export { Favicon }
