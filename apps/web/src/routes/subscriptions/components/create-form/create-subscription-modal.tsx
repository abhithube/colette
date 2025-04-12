import { EditStep } from './edit-step'
import { SearchStep } from './search-step'
import { SelectStep } from './select-step'
import type { Feed, FeedDetected } from '@colette/core'
import { Dialog } from '@colette/ui'
import { Sidebar } from '@colette/ui'
import { Plus } from 'lucide-react'
import { useState } from 'react'

enum Step {
  Search = 0,
  Select = 1,
  Edit = 2,
}

export const CreateSubscriptionModal = () => {
  const [step, setStep] = useState(Step.Search)
  const [detectedFeeds, setDetectedFeeds] = useState<FeedDetected[] | null>(
    null,
  )
  const [selectedFeed, setSelectedFeed] = useState<Feed | null>(null)

  return (
    <Dialog.Root>
      <Dialog.Trigger asChild>
        <Sidebar.GroupAction>
          <Plus />
        </Sidebar.GroupAction>
      </Dialog.Trigger>
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
      <Dialog.Context>
        {(dialogProps) =>
          step === Step.Edit &&
          selectedFeed && (
            <EditStep
              feed={selectedFeed}
              onClose={() => {
                dialogProps.setOpen(false)

                setStep(Step.Search)
                setDetectedFeeds(null)
                setSelectedFeed(null)
              }}
              onBack={() => {
                setStep(detectedFeeds ? Step.Select : Step.Search)
              }}
            />
          )
        }
      </Dialog.Context>
    </Dialog.Root>
  )
}
