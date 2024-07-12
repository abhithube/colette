import { Favicon } from '@/components/favicon'
import { CardTitle } from '@/components/ui/card'
import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from '@/components/ui/tooltip'
import { formatRelativeDate } from '@/lib/utils'

type TitleProps = {
	title: string
	link: string
}

export function EntryTitle({ title, link }: TitleProps) {
	return (
		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					<CardTitle className="truncate text-left text-lg leading-tight">
						<a href={link} target="_blank" rel="noreferrer">
							{title}
						</a>
					</CardTitle>
				</TooltipTrigger>
				<TooltipContent>{title}</TooltipContent>
			</Tooltip>
		</TooltipProvider>
	)
}

type ThumbnailProps = {
	src?: string | null
	alt: string
}

export function EntryThumbnail({ src, alt }: ThumbnailProps) {
	return (
		<img
			className="aspect-video w-full bg-background object-cover"
			src={src ?? 'https://placehold.co/320x180/black/black'}
			alt={alt}
			loading="lazy"
		/>
	)
}

type PublishedProps = {
	publishedAt: string
}

export function EntryPublished({ publishedAt }: PublishedProps) {
	return (
		<span className="font-semibold text-muted-foreground text-xs">
			{formatRelativeDate(Date.parse(publishedAt))}
		</span>
	)
}

type AuthorProps = {
	author?: string | null
	link: string
}

export function EntryAuthor({ author, link }: AuthorProps) {
	return (
		<div className="flex space-x-1">
			<Favicon domain={new URL(link).hostname} />
			<span className="font-semibold text-muted-foreground text-xs">
				{author ?? 'Anonymous'}
			</span>
		</div>
	)
}
