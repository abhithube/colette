import { cn } from './utils'
import { ark, type HTMLArkProps } from '@ark-ui/react'

export type SkeletonProps = HTMLArkProps<'div'>

export const Skeleton = ({ className, ...props }: SkeletonProps) => {
  return (
    <ark.div
      data-scope="skeleton"
      className={cn('bg-accent animate-pulse rounded-md', className)}
      {...props}
    />
  )
}
