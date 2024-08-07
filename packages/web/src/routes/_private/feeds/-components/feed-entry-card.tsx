import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import type { Entry } from '@colette/openapi'
import {
	EntryAuthor,
	EntryPublished,
	EntryThumbnail,
	EntryTitle,
} from '../../-components/entry-parts'

type Props = {
	entry: Entry
}

export function FeedEntryCard({ entry }: Props) {
	return (
		<Card className="overflow-hidden shadow-md">
			<EntryThumbnail src={entry.thumbnailUrl} alt={entry.title} />
			<div className="flex flex-col pb-2">
				<CardHeader>
					<EntryTitle title={entry.title} link={entry.link} />
				</CardHeader>
				<CardContent className="flex justify-between">
					<div className="flex h-4 space-x-2">
						<EntryAuthor author={entry.author} link={entry.link} />
						<Separator
							className="bg-muted-foreground/50"
							orientation="vertical"
						/>
						{entry.publishedAt && (
							<EntryPublished publishedAt={entry.publishedAt} />
						)}
					</div>
				</CardContent>
			</div>
		</Card>
	)
}
