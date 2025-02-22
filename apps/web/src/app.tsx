import { AuthLayout } from './auth-layout'
import { CollectionPage } from './routes/collections/id'
import { StashPage } from './routes/collections/stash'
import { HomePage } from './routes/feeds'
import { ArchivedPage } from './routes/feeds/archived'
import { FeedPage } from './routes/feeds/id'
import { LoginPage } from './routes/login'
import type { FC } from 'react'
import { Route, Switch } from 'wouter'

export const App: FC = () => {
  return (
    <Switch>
      <Route path="/login" component={LoginPage} />
      <Route path="/" nest>
        <AuthLayout>
          <Route path="/" component={HomePage} />
          <Route path="/archived" component={ArchivedPage} />
          <Route path="/stash" component={StashPage} />
          <Route path="/feeds/:id" component={FeedPage} />
          <Route path="/collections/:id" component={CollectionPage} />
        </AuthLayout>
      </Route>
    </Switch>
  )
}
