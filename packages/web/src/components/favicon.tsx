import { cn } from '@/lib/utils'

export interface FaviconProps extends React.HTMLAttributes<HTMLDivElement> {
  domain: string
}

const Favicon = ({ className, domain, ...props }: FaviconProps) => {
  return (
    <img
      className={cn('min-h-4 min-w-4', className)}
      src={`https://icons.duckduckgo.com/ip3/${domain}.ico`}
      width={16}
      height={16}
      {...props}
      alt={domain}
    />
  )
}

export { Favicon }
