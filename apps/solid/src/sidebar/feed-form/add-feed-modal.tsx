import Plus from 'lucide-solid/icons/plus'
import { Match, Switch, createSignal } from 'solid-js'
import { Dialog, DialogContent, DialogTrigger } from '~/components/ui/dialog'
import { SidebarGroupAction } from '~/components/ui/sidebar'
import { SearchStep } from './search-step'

enum Step {
  Search = 0,
  Select = 1,
  Edit = 2,
}

export function AddFeedModal() {
  const [isOpen, setOpen] = createSignal(false)
  const [step, setStep] = createSignal(Step.Search)

  return (
    <Dialog open={isOpen()} onOpenChange={setOpen}>
      <DialogTrigger as={SidebarGroupAction}>
        <Plus />
      </DialogTrigger>
      <DialogContent class="gap-6">
        <Switch>
          <Match when={step() === Step.Search}>
            <SearchStep
              onNext={(res) => {
                if (Array.isArray(res)) {
                  setStep(Step.Select)
                } else {
                  setStep(Step.Edit)
                }
              }}
            />
          </Match>
        </Switch>
      </DialogContent>
    </Dialog>
  )
}
