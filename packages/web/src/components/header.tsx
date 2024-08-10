import { cn } from '@/lib/utils'
import React from 'react'
import { Button, type ButtonProps } from './ui/button'

const Header = React.forwardRef<HTMLElement, React.HTMLAttributes<HTMLElement>>(
  ({ className, ...props }, ref) => (
    <header
      ref={ref}
      className={cn(
        'sticky top-0 flex w-full items-center justify-between bg-background p-8',
        className,
      )}
      {...props}
    />
  ),
)
Header.displayName = 'Header'

const HeaderTitle = React.forwardRef<
  HTMLHeadingElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h1
    ref={ref}
    className={cn('truncate font-medium text-3xl', className)}
    {...props}
  />
))
HeaderTitle.displayName = 'HeaderTitle'

const HeaderActionGroup = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn('flex items-center space-x-2', className)}
    {...props}
  />
))
HeaderActionGroup.displayName = 'HeaderActionGroup'

const HeaderActionItem = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, asChild = false, ...props }, ref) => (
    <Button
      ref={ref}
      className={cn('flex items-center space-x-2', className)}
      variant="secondary"
      asChild={asChild}
      {...props}
    />
  ),
)
HeaderActionItem.displayName = 'HeaderActionItem'

export { Header, HeaderTitle, HeaderActionGroup, HeaderActionItem }
