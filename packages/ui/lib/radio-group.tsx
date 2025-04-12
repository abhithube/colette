import { cn } from './utils'
import { RadioGroup } from '@ark-ui/react'
import { Circle } from 'lucide-react'

export type RootProps = RadioGroup.RootProps

export const Root = ({ className, ...props }: RootProps) => {
  return <RadioGroup.Root className={cn('grid gap-3', className)} {...props} />
}

export type ItemProps = RadioGroup.ItemProps

export const Item = ({ className, ...props }: ItemProps) => {
  return (
    <RadioGroup.Item
      className={cn(
        'border-input text-primary focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 aspect-square size-4 shrink-0 rounded-full border shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      {...props}
    >
      <RadioGroup.Indicator className="relative flex items-center justify-center">
        <Circle className="fill-primary absolute top-1/2 left-1/2 size-2 -translate-x-1/2 -translate-y-1/2" />
      </RadioGroup.Indicator>
    </RadioGroup.Item>
  )
}
