import { EditStep } from './edit-step'
import { SearchStep } from './search-step'
import type { BookmarkScraped } from '@colette/core'
import { CREATE_BOOKMARK_FORM, SCRAPE_BOOKMARK_FORM } from '@colette/form'
import { Button, Dialog, Steps } from '@colette/ui'
import { useState } from 'react'

const items = [
  {
    id: SCRAPE_BOOKMARK_FORM,
    description: 'Search for a bookmark by URL',
    value: 'Search',
  },
  {
    id: CREATE_BOOKMARK_FORM,
    description: 'Set bookmark metadata',
    value: 'Submit',
  },
]

export const CreateBookmarkModal = (props: { close: () => void }) => {
  const steps = Steps.useSteps({
    count: items.length,
    linear: true,
  })

  const item = items[steps.value]

  const [scraped, setScraped] = useState<BookmarkScraped | null>(null)

  return (
    <Dialog.Content>
      <Dialog.Header>
        <Dialog.Title>Add Bookmark</Dialog.Title>
        <Dialog.Description>{item.description}</Dialog.Description>
      </Dialog.Header>

      <Steps.Provider className="grid gap-4" value={steps}>
        <Steps.List className="px-16">
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
              onNext={(scraped) => {
                setScraped(scraped)

                steps.goToNextStep()
              }}
            />
          </Steps.Content>
        )}

        {steps.value === 1 && scraped && (
          <Steps.Content index={steps.value}>
            <EditStep
              formId={item.id}
              bookmark={scraped}
              onClose={() => {
                props.close()

                steps.resetStep()

                setScraped(null)
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
