import type { ComponentPropsWithRef, FC } from 'react'
import { cn } from '~/lib/utils'

const FormDescription: FC<ComponentPropsWithRef<'p'>> = ({
  className,
  ...props
}) => {
  return (
    <p className={cn('text-muted-foreground text-sm', className)} {...props} />
  )
}
FormDescription.displayName = 'FormDescription'

const FormMessage: FC<ComponentPropsWithRef<'p'>> = ({
  className,
  children,
  ...props
}) => {
  if (!children) return null

  return (
    <p
      className={cn('font-medium text-destructive text-sm', className)}
      {...props}
    >
      {children}
    </p>
  )
}
FormMessage.displayName = 'FormMessage'

export { FormDescription, FormMessage }
