import { Button, type ButtonProps } from '@/components/ui/button'
import { forwardRef } from 'react'

export const SidebarButton = forwardRef<HTMLButtonElement, ButtonProps>(
	({ children, ...props }, ref) => {
		return (
			<Button
				className="flex w-full hover:bg-muted/50 items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-primary text-sm"
				ref={ref}
				variant="ghost"
				{...props}
			>
				{children}
			</Button>
		)
	},
)
