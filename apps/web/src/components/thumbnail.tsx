import { Skeleton } from './ui/skeleton'
import { FC, useState } from 'react'
import { cn } from '~/lib/utils'

export const Thumbnail: FC<React.ImgHTMLAttributes<HTMLImageElement>> = (
  props,
) => {
  const [isLoading, setLoading] = useState(true)

  return (
    <div className="aspect-video object-cover">
      {isLoading && <Skeleton className="size-full" />}
      <img
        className={cn(isLoading && 'opacity-0', props.className)}
        src={props.src ?? 'https://placehold.co/320x180/black/black'}
        loading="lazy"
        onLoad={() => setLoading(false)}
        {...props}
      />
    </div>
  )
}
