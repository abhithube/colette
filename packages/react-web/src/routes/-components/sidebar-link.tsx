import { Link, type LinkOptions } from '@tanstack/react-router'

type Props = {
	children: React.ReactNode
}

export const SidebarLink = ({ children, ...props }: LinkOptions & Props) => (
	<Link
		className="flex h-10 w-full items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-primary text-sm"
		inactiveProps={{ className: 'hover:bg-muted/50' }}
		activeProps={{ className: 'bg-muted text-secondary' }}
		{...props}
	>
		{children}
	</Link>
)
