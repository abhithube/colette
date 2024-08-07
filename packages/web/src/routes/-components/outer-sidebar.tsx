import { Dialog, DialogTrigger } from '@/components/ui/dialog'
import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from '@/components/ui/tooltip'
import type { Profile } from '@colette/openapi'
import { Bookmark, Home, Rss, Search, Settings, User } from 'lucide-react'
import { useState } from 'react'
import { ProfileModal } from './profile-modal'
import { SettingsModal } from './settings-modal'
import { SidebarButton } from './sidebar-button'
import { SidebarLink } from './sidebar-link'

type Props = {
	profile: Profile
}

export const OuterSidebar = ({ profile }: Props) => {
	const [isProfileModalOpen, setProfileModalOpen] = useState(false)
	const [isSettingsModalOpen, setSettingsModalOpen] = useState(false)

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
						<SidebarLink to="/bookmarks" activeOptions={{ exact: false }}>
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
			<Dialog open={isProfileModalOpen} onOpenChange={setProfileModalOpen}>
				<TooltipProvider>
					<Tooltip>
						<TooltipTrigger asChild>
							<DialogTrigger asChild>
								<SidebarButton>
									<User className="h-5 w-5 shrink-0" />
								</SidebarButton>
							</DialogTrigger>
						</TooltipTrigger>
						<TooltipContent side="right">Profile</TooltipContent>
					</Tooltip>
				</TooltipProvider>
				<ProfileModal
					profile={profile}
					close={() => setProfileModalOpen(false)}
				/>
			</Dialog>
			<Dialog open={isSettingsModalOpen} onOpenChange={setSettingsModalOpen}>
				<TooltipProvider>
					<Tooltip>
						<TooltipTrigger asChild>
							<DialogTrigger asChild>
								<SidebarButton>
									<Settings className="h-5 w-5 shrink-0" />
								</SidebarButton>
							</DialogTrigger>
						</TooltipTrigger>
						<TooltipContent side="right">Settings</TooltipContent>
					</Tooltip>
				</TooltipProvider>
				<SettingsModal close={() => setSettingsModalOpen(false)} />
			</Dialog>
		</nav>
	)
}
