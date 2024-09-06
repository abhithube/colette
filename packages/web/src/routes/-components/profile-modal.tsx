import { Icon } from '@/components/icon'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter, CardTitle } from '@/components/ui/card'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { cn } from '@/lib/utils'
import { Dialog, IconButton } from '@colette/components'
import type { Profile } from '@colette/core'
import { listProfilesOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { CheckCircle, Plus, XIcon } from 'lucide-react'
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
    <Dialog.Content>
      <Dialog.Title>Profile</Dialog.Title>
      <Dialog.Description>Select a profile</Dialog.Description>
      <RadioGroup className="grid grid-cols-3" value={selected}>
        {profiles.data.map((p) => (
          <div key={p.id}>
            <RadioGroupItem id={p.id} className="hidden" value={p.id} />
            <Card
              className={cn(
                'w-28 cursor-pointer p-4',
                selected === p.id && 'border-primary',
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
                      <Icon value={CheckCircle} />
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
            <Icon value={Plus} />
          </Button>
          <span className="text-muted-foreground text-sm">Create new</span>
        </div>
      </RadioGroup>
      <Button disabled={selected === profile.id}>Select</Button>
      <Dialog.CloseTrigger asChild position="absolute" top="2" right="2">
        <IconButton aria-label="Close Dialog" variant="ghost" size="sm">
          <XIcon />
        </IconButton>
      </Dialog.CloseTrigger>
    </Dialog.Content>
  )
}
