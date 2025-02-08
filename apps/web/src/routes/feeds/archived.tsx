import { EntryList } from './components/entry-list'
import { type FC, useEffect } from 'react'

export const ArchivedPage: FC = () => {
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">Archived</h1>
      </div>
      <main>
        <EntryList query={{ hasRead: true }} />
      </main>
    </>
  )
}
