import { EditStep } from './edit-step'
import { SearchStep } from './search-step'
import { SelectStep } from './select-step'
import { Feed, FeedDetected } from '@colette/core/types'
import {
  CREATE_SUBSCRIPTION_FORM,
  DETECT_FEEDS_FORM,
  SCRAPE_FEED_FORM,
} from '@colette/form'
import { Button, Dialog, Steps } from '@colette/ui'
import { useState } from 'react'

const items = [
  {
    id: DETECT_FEEDS_FORM,
    description: 'Search for a feed by URL',
    value: 'Search',
  },
  {
    id: SCRAPE_FEED_FORM,
    description: 'Select a feed to subscribe to',
    value: 'Select',
  },
  {
    id: CREATE_SUBSCRIPTION_FORM,
    description: 'Set subscription metadata',
    value: 'Submit',
  },
]

export const CreateSubscriptionModal = (props: { close: () => void }) => {
  const steps = Steps.useSteps({
    count: items.length,
    linear: true,
  })

  const item = items[steps.value]

  const [detectedFeeds, setDetectedFeeds] = useState<FeedDetected[] | null>(
    null,
  )
  const [selectedFeed, setSelectedFeed] = useState<Feed | null>(null)

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title>Subscribe to Feed</Dialog.Title>
        <Dialog.Description>{item.description}</Dialog.Description>
      </Dialog.Header>

      <Steps.Provider className="grid gap-4" value={steps}>
        <Steps.List>
          {items.map((item, index) => {
            return (
              <Steps.Item key={index} index={index}>
                <Steps.Trigger>
                  <Steps.Indicator>{index + 1}</Steps.Indicator>
                  <span>{item.value}</span>
                </Steps.Trigger>
                <Steps.Separator />
              </Steps.Item>
            )
          })}
        </Steps.List>

        {steps.value === 0 && (
          <Steps.Content index={steps.value}>
            <SearchStep
              formId={item.id}
              onNext={(detected) => {
                setDetectedFeeds(detected)

                steps.goToNextStep()
              }}
            />
          </Steps.Content>
        )}

        {steps.value === 1 && detectedFeeds && (
          <Steps.Content index={steps.value}>
            <SelectStep
              formId={item.id}
              feeds={detectedFeeds}
              onNext={(feed) => {
                setSelectedFeed(feed)

                steps.goToNextStep()
              }}
            />
          </Steps.Content>
        )}

        {steps.value === 2 && selectedFeed && (
          <Steps.Content index={steps.value}>
            <EditStep
              formId={item.id}
              feed={selectedFeed}
              onClose={() => {
                props.close()

                steps.resetStep()

                setDetectedFeeds(null)
                setSelectedFeed(null)
              }}
            />
          </Steps.Content>
        )}

        <Dialog.Footer>
          <Steps.PrevTrigger asChild>
            <Button variant="outline" disabled={!steps.hasPrevStep}>
              Back
            </Button>
          </Steps.PrevTrigger>
          <Button type="submit" form={item.id}>
            {item.value}
          </Button>
        </Dialog.Footer>
      </Steps.Provider>
    </Dialog.Content>
  )
}
