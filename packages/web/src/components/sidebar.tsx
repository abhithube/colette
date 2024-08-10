import { Button, type ButtonProps } from '@/components/ui/button'
import { Link, type LinkProps } from '@tanstack/react-router'
import React from 'react'

const SidebarButton = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ children, ...props }, ref) => {
    return (
      <Button
        ref={ref}
        className="flex w-full items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-sm hover:bg-muted/50"
        variant="ghost"
        {...props}
      >
        {children}
      </Button>
    )
  },
)
SidebarButton.displayName = 'SidebarButton'

const SidebarLink = React.forwardRef<HTMLAnchorElement, LinkProps>(
  ({ children, ...props }, ref) => (
    <Link
      ref={ref}
      className="flex h-10 w-full items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-sm"
      inactiveProps={{ className: 'hover:bg-muted/50' }}
      activeProps={{ className: 'bg-muted' }}
      {...props}
    >
      {children}
    </Link>
  ),
)
SidebarLink.displayName = 'SidebarLink'

export { SidebarButton, SidebarLink }
