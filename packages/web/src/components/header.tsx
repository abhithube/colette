import { cn } from '@/lib/utils'
import React from 'react'

const Header = React.forwardRef<
	HTMLDivElement,
	React.HTMLAttributes<HTMLElement>
>(({ className, ...props }, ref) => (
	<header
		ref={ref}
		className={cn('sticky top-0 w-full bg-background p-8', className)}
		{...props}
	/>
))
Header.displayName = 'Header'

const HeaderTitle = React.forwardRef<
	HTMLDivElement,
	React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
	<h1
		ref={ref}
		className={cn('truncate font-medium text-3xl', className)}
		{...props}
	/>
))
HeaderTitle.displayName = 'HeaderTitle'

export { Header, HeaderTitle }
