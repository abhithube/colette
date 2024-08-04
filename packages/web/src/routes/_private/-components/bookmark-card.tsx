import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import type { Bookmark } from '@colette/openapi'
import {
	EntryAuthor,
	EntryPublished,
	EntryThumbnail,
	EntryTitle,
} from './entry-parts'

type Props = {
	bookmark: Bookmark
}

export function BookmarkCard({ bookmark }: Props) {
	return (
		<Card className="overflow-hidden shadow-md">
			<EntryThumbnail src={bookmark.thumbnailUrl} alt={bookmark.title} />
			<div className="flex flex-col pb-2">
				<CardHeader>
					<EntryTitle title={bookmark.title} link={bookmark.link} />
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
		</Card>
	)
}
