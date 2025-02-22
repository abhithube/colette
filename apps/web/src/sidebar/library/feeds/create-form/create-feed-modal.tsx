import { EditStep } from './edit-step'
import { SearchStep } from './search-step'
import { SelectStep } from './select-step'
import type { FeedDetected, FeedProcessed } from '@colette/core'
import { Plus } from 'lucide-react'
import { type FC, useState } from 'react'
import { Dialog } from '~/components/dialog'
import { DialogTrigger } from '~/components/ui/dialog'
import { SidebarGroupAction } from '~/components/ui/sidebar'

enum Step {
  Search = 0,
  Select = 1,
  Edit = 2,
}

export const CreateFeedModal: FC = () => {
  const [step, setStep] = useState(Step.Search)
  const [detectedFeeds, setDetectedFeeds] = useState<FeedDetected[] | null>(
    null,
  )
  const [selectedFeed, setSelectedFeed] = useState<FeedProcessed | null>(null)

  return (
    <Dialog>
      {(close) => (
        <>
          <DialogTrigger asChild>
            <SidebarGroupAction>
              <Plus />
            </SidebarGroupAction>
          </DialogTrigger>
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
                close()

                setStep(Step.Search)
                setDetectedFeeds(null)
                setSelectedFeed(null)
              }}
              onBack={() => {
                setStep(detectedFeeds ? Step.Select : Step.Search)
              }}
            />
          )}
        </>
      )}
    </Dialog>
  )
}
