import { Icon } from '@/components/icon'
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
import { SidebarButton, SidebarLink } from '../../components/sidebar'
import { ProfileModal } from './profile-modal'
import { SettingsModal } from './settings-modal'

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
							<Icon size="lg" value={Home} />
						</SidebarLink>
					</TooltipTrigger>
					<TooltipContent side="right">Home</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger>
						<SidebarLink to="/feeds" activeOptions={{ exact: false }}>
							<Icon size="lg" value={Rss} />
						</SidebarLink>
					</TooltipTrigger>
					<TooltipContent side="right">Feed</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger>
						<SidebarLink to="/bookmarks" activeOptions={{ exact: false }}>
							<Icon size="lg" value={Bookmark} />
						</SidebarLink>
					</TooltipTrigger>
					<TooltipContent side="right">Bookmarks</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger asChild>
						<SidebarButton>
							<Icon size="lg" value={Search} />
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
									<Icon size="lg" value={User} />
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
									<Icon size="lg" value={Settings} />
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
