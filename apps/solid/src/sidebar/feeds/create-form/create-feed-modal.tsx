import { EditStep } from './edit-step'
import { SearchStep } from './search-step'
import { SelectStep } from './select-step'
import type { FeedDetected, FeedProcessed } from '@colette/core'
import Plus from 'lucide-solid/icons/plus'
import { type Component, Match, Show, Switch, createSignal } from 'solid-js'
import { Dialog, DialogContent, DialogTrigger } from '~/components/ui/dialog'
import { SidebarGroupAction } from '~/components/ui/sidebar'

enum Step {
  Search = 0,
  Select = 1,
  Edit = 2,
}

export const CreateFeedModal: Component = () => {
  const [open, setOpen] = createSignal(false)
  const [step, setStep] = createSignal(Step.Search)
  const [detectedFeeds, setDetectedFeeds] = createSignal<FeedDetected[] | null>(
    null,
  )
  const [selectedFeed, setSelectedFeed] = createSignal<FeedProcessed | null>(
    null,
  )

  return (
    <Dialog open={open()} onOpenChange={setOpen}>
      <DialogTrigger as={SidebarGroupAction}>
        <Plus />
      </DialogTrigger>
      <DialogContent class="gap-6">
        <Switch>
          <Match when={step() === Step.Search}>
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
          </Match>
          <Match when={step() === Step.Select}>
            <Show when={detectedFeeds()}>
              {(feeds) => (
                <SelectStep
                  feeds={feeds()}
                  onNext={(feed) => {
                    setSelectedFeed(feed)
                    setStep(Step.Edit)
                  }}
                  onBack={() => setStep(Step.Search)}
                />
              )}
            </Show>
          </Match>
          <Match when={step() === Step.Edit}>
            <Show when={selectedFeed()}>
              {(feed) => (
                <EditStep
                  feed={feed()}
                  onClose={() => {
                    setOpen(false)
                    setStep(Step.Search)
                    setDetectedFeeds(null)
                    setSelectedFeed(null)
                  }}
                  onBack={() => {
                    setStep(detectedFeeds() ? Step.Select : Step.Search)
                  }}
                />
              )}
            </Show>
          </Match>
        </Switch>
      </DialogContent>
    </Dialog>
  )
}
