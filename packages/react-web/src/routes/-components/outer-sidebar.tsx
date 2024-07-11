import { Bookmark, Home, Rss, Settings, User } from 'lucide-react'
import { SidebarButton } from './sidebar-button'
import { SidebarLink } from './sidebar-link'

export const OuterSidebar = () => {
	return (
		<nav className="flex h-full flex-col space-y-4 border-r p-4">
			<SidebarLink to="/">
				<Home className="h-5 w-5 shrink-0" />
			</SidebarLink>
			<SidebarLink to="/feeds">
				<Rss className="h-5 w-5 shrink-0" />
			</SidebarLink>
			<SidebarLink to="/collections">
				<Bookmark className="h-5 w-5 shrink-0" />
			</SidebarLink>
			<div className="flex-grow" />
			<SidebarButton>
				<User className="h-5 w-5 shrink-0" />
			</SidebarButton>
			<SidebarButton>
				<Settings className="h-5 w-5 shrink-0" />
			</SidebarButton>
		</nav>
	)
}
