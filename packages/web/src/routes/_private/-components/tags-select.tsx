import { Icon } from '@/components/icon'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '@/components/ui/command'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '@/components/ui/popover'
import type { Tag } from '@colette/openapi'
import { ChevronsUpDown, X } from 'lucide-react'
import { useState } from 'react'

type Props = {
	options: Tag[]
	value: string[]
	onChange: (value: string[]) => void
}

export function TagsSelect({ options, value: currentValue, onChange }: Props) {
	const [open, setOpen] = useState(false)

	const filtered = options.filter(
		(option) => !currentValue.includes(option.title),
	)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button
					variant="outline"
					role="combobox"
					aria-expanded={open}
					className="h-full min-h-14 w-full justify-between"
					onClick={() => {
						setOpen(!open)
					}}
				>
					<div className="flex flex-wrap gap-2">
						{currentValue.map((item) => (
							<Badge
								key={item}
								className="rounded-md p-2"
								onClick={() => {
									setOpen(true)
								}}
							>
								{item}
								<div className="ml-1 outline-none ring-offset-background focus:ring-2 focus:ring-ring focus:ring-offset-2">
									<Icon
										value={X}
										role="button"
										className="text-muted-foreground"
										onKeyDown={(e) => {
											if (e.key === 'Enter' || e.key === 'Backspace') {
												onChange(currentValue.filter((val) => val !== item))
											}
										}}
										onClick={() => {
											onChange(currentValue.filter((val) => val !== item))
										}}
									/>
								</div>
							</Badge>
						))}
					</div>
					<Icon className="opacity-50" value={ChevronsUpDown} />
				</Button>
			</PopoverTrigger>
			<PopoverContent className="min-w-[var(--radix-popover-trigger-width)] p-0">
				<Command>
					<CommandInput placeholder="Search tags..." />
					<CommandList>
						<CommandEmpty>No item found.</CommandEmpty>
						<CommandGroup className="max-h-64 overflow-auto">
							{filtered.map((tag) => (
								<CommandItem
									key={tag.title}
									onSelect={() => {
										onChange([...currentValue, tag.title].toSorted())
										setOpen(true)
									}}
								>
									{tag.title}
								</CommandItem>
							))}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	)
}
