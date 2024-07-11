import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from '@/components/ui/tooltip'
import { Bookmark, Home, Rss, Search, Settings, User } from 'lucide-react'
import { SidebarButton } from './sidebar-button'
import { SidebarLink } from './sidebar-link'

export const OuterSidebar = () => {
	return (
		<nav className="flex h-full flex-col space-y-4 border-r p-4">
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger>
						<SidebarLink to="/">
							<Home className="h-5 w-5 shrink-0" />
						</SidebarLink>
					</TooltipTrigger>
					<TooltipContent side="right">Home</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger>
						<SidebarLink to="/feeds" activeOptions={{ exact: false }}>
							<Rss className="h-5 w-5 shrink-0" />
						</SidebarLink>
					</TooltipTrigger>
					<TooltipContent side="right">Feed</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger>
						<SidebarLink to="/collections" activeOptions={{ exact: false }}>
							<Bookmark className="h-5 w-5 shrink-0" />
						</SidebarLink>
					</TooltipTrigger>
					<TooltipContent side="right">Bookmarks</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger asChild>
						<SidebarButton>
							<Search className="h-5 w-5 shrink-0" />
						</SidebarButton>
					</TooltipTrigger>
					<TooltipContent side="right">Search</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<div className="flex-grow" />
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger asChild>
						<SidebarButton>
							<User className="h-5 w-5 shrink-0" />
						</SidebarButton>
					</TooltipTrigger>
					<TooltipContent side="right">Profile</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger asChild>
						<SidebarButton>
							<Settings className="h-5 w-5 shrink-0" />
						</SidebarButton>
					</TooltipTrigger>
					<TooltipContent side="right">Settings</TooltipContent>
				</Tooltip>
			</TooltipProvider>
		</nav>
	)
}
