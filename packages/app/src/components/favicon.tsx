export interface FaviconProps extends React.HTMLAttributes<HTMLDivElement> {
  domain: string
}

const Favicon = ({ domain, ...props }: FaviconProps) => {
  return (
    <img
      src={`https://icons.duckduckgo.com/ip3/${domain}.ico`}
      width={16}
      height={16}
      {...props}
      alt={domain}
    />
  )
}

export { Favicon }
