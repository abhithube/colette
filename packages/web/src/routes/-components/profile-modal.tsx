import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter, CardTitle } from '@/components/ui/card'
import {
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { cn } from '@/lib/utils'
import type { Profile } from '@colette/openapi'
import { listProfilesOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { CheckCircle, Plus } from 'lucide-react'
import { useState } from 'react'
import { Route } from '../_private'

type Props = {
	profile: Profile
	close: () => void
}

export function ProfileModal({ profile }: Props) {
	const context = Route.useRouteContext()

	const { data: profiles } = useQuery(listProfilesOptions(context.api))

	const [selected, setSelected] = useState(profile.id)

	if (!profiles) return

	return (
		<DialogContent className="max-w-[425px]">
			<DialogHeader>
				<DialogTitle>Profile</DialogTitle>
				<DialogDescription>Select a profile</DialogDescription>
			</DialogHeader>
			<RadioGroup className="grid grid-cols-3" value={selected}>
				{profiles.data.map((p) => (
					<div key={p.id}>
						<RadioGroupItem id={p.id} className="hidden" value={p.id} />
						<Card
							className={cn(
								'w-28 cursor-pointer p-4',
								selected === p.id && 'border-secondary',
							)}
							onClick={() => setSelected(p.id)}
						>
							<CardContent className="flex flex-col items-center justify-center space-y-2 p-0">
								<Avatar>
									<AvatarImage src={p.imageUrl ?? undefined} />
									<AvatarFallback>{p.title[0]}</AvatarFallback>
								</Avatar>
								<CardTitle className="text-sm">{p.title}</CardTitle>
								<CardFooter className="p-0 text-muted-foreground text-xs italic">
									<span className="h-[1lh]">
										{p.id === profile.id ? (
											'Active'
										) : p.id === selected ? (
											<CheckCircle className="h-4 w-4 shrink-0 text-secondary" />
										) : (
											''
										)}
									</span>
								</CardFooter>
							</CardContent>
						</Card>
					</div>
				))}
				<div className="flex flex-col items-center justify-center space-y-2">
					<Button variant="outline" className="h-10 w-10 rounded-full">
						<Plus className="h-4 w-4 shrink-0" />
					</Button>
					<span className="text-muted-foreground text-sm">Create new</span>
				</div>
			</RadioGroup>
			<DialogFooter>
				<Button disabled={selected === profile.id}>Select</Button>
			</DialogFooter>
		</DialogContent>
	)
}
