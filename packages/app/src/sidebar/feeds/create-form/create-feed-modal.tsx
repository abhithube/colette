import { EditStep } from './edit-step'
import { SearchStep } from './search-step'
import { SelectStep } from './select-step'
import type { FeedDetected, FeedProcessed } from '@colette/core'
import {
  Dialog,
  DialogContent,
  DialogTrigger,
} from '@colette/react-ui/components/ui/dialog'
import { SidebarGroupAction } from '@colette/react-ui/components/ui/sidebar'
import { Plus } from 'lucide-react'
import { type FC, useState } from 'react'

enum Step {
  Search = 0,
  Select = 1,
  Edit = 2,
}

export const CreateFeedModal: FC = () => {
  const [isOpen, setOpen] = useState(false)
  const [step, setStep] = useState(Step.Search)
  const [detectedFeeds, setDetectedFeeds] = useState<FeedDetected[] | null>(
    null,
  )
  const [selectedFeed, setSelectedFeed] = useState<FeedProcessed | null>(null)

  return (
    <Dialog open={isOpen} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <SidebarGroupAction>
          <Plus />
        </SidebarGroupAction>
      </DialogTrigger>
      <DialogContent className="gap-6">
        {step === Step.Search && (
          <SearchStep
            onNext={(res) => {
              if (Array.isArray(res)) {
                setDetectedFeeds(res)
                setStep(Step.Select)
              } else {
                setSelectedFeed(res)
                setStep(Step.Edit)
              }
            }}
          />
        )}
        {step === Step.Select && detectedFeeds && (
          <SelectStep
            feeds={detectedFeeds}
            onNext={(feed) => {
              setSelectedFeed(feed)
              setStep(Step.Edit)
            }}
            onBack={() => setStep(Step.Search)}
          />
        )}
        {step === Step.Edit && selectedFeed && (
          <EditStep
            feed={selectedFeed}
            onClose={() => {
              setOpen(false)
              setStep(Step.Search)
              setDetectedFeeds(null)
              setSelectedFeed(null)
            }}
            onBack={() => {
              setStep(detectedFeeds ? Step.Select : Step.Search)
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}
