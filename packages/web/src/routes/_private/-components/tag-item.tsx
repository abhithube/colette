import { SidebarLink } from '@/components/sidebar'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { Tag } from '@colette/openapi'

type Props = {
  tag: Tag
  type: 'bookmark' | 'feed'
}

export function TagItem({ tag, type }: Props) {
  return (
    <SidebarLink
      to={type === 'bookmark' ? '/bookmarks/tags/$id' : '/feeds/tags/$id'}
      params={{
        id: tag.id,
      }}
    >
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger className="grow truncate text-left">
            {tag.title}
          </TooltipTrigger>
          <TooltipContent>{tag.title}</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <div className="flex w-[3ch] shrink-0 justify-center">
        <span className={cn('text-muted-foreground tabular-nums')}>
          {type === 'bookmark' ? tag.bookmarkCount : tag.feedCount}
        </span>
      </div>
    </SidebarLink>
  )
}
