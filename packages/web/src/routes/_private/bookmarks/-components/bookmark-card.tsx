import { Icon } from '@/components/icon'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Dialog } from '@/components/ui/dialog'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Separator } from '@/components/ui/separator'
import type { Bookmark } from '@colette/openapi'
import { MoreHorizontal } from 'lucide-react'
import { useState } from 'react'
import {
	EntryAuthor,
	EntryPublished,
	EntryThumbnail,
	EntryTitle,
} from '../../-components/entry-parts'
import { EditBookmarkModal } from './edit-bookmark-modal'

type Props = {
	bookmark: Bookmark
}

export function BookmarkCard({ bookmark }: Props) {
	const [isEditModalOpen, setEditModalOpen] = useState(false)

	return (
		<Card className="overflow-hidden shadow-md">
			<EntryThumbnail src={bookmark.thumbnailUrl} alt={bookmark.title} />
			<div className="flex flex-col pb-2">
				<CardHeader>
					<EntryTitle title={bookmark.title} link={bookmark.link} />
					<DropdownMenu>
						<DropdownMenuTrigger>
							<Icon className="text-muted-foreground" value={MoreHorizontal} />
						</DropdownMenuTrigger>
						<DropdownMenuContent className="w-56">
							<DropdownMenuItem asChild>
								<a href={bookmark.link} target="_blank" rel="noreferrer">
									Open in new tab
								</a>
							</DropdownMenuItem>
							<DropdownMenuItem onClick={() => setEditModalOpen(true)}>
								Edit
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				</CardHeader>
				<CardContent className="flex justify-between">
					<div className="flex h-4 space-x-2">
						<EntryAuthor author={bookmark.author} link={bookmark.link} />
						<Separator
							className="bg-muted-foreground/50"
							orientation="vertical"
						/>
						{bookmark.publishedAt && (
							<EntryPublished publishedAt={bookmark.publishedAt} />
						)}
					</div>
				</CardContent>
			</div>
			<Dialog open={isEditModalOpen} onOpenChange={setEditModalOpen}>
				<EditBookmarkModal
					bookmark={bookmark}
					close={() => setEditModalOpen(false)}
				/>
			</Dialog>
		</Card>
	)
}
