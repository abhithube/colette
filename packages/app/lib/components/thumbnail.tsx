import { Skeleton } from '@colette/ui'
import { cn } from '@colette/ui/utils'
import { useState } from 'react'

export const Thumbnail = (props: React.ImgHTMLAttributes<HTMLImageElement>) => {
  const [isLoading, setLoading] = useState(true)

  return (
    <div className="relative aspect-video">
      {isLoading && (
        <div className="absolute inset-0">
          <Skeleton className="size-full" />
        </div>
      )}
      <img
        className={cn(
          'size-full object-cover',
          isLoading && 'invisible',
          props.className,
        )}
        src={props.src ?? 'https://placehold.co/320x180/black/black'}
        loading="lazy"
        onLoad={() => setLoading(false)}
        {...props}
      />
    </div>
  )
}
