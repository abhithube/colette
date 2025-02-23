import { AuthLayout } from './auth-layout'
import { StashPage } from './routes/bookmarks'
import { FeedsPage } from './routes/feeds'
import { ArchivedPage } from './routes/feeds/archived'
import { FeedPage } from './routes/feeds/id'
import { HomePage } from './routes/home'
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
          <Route path="/feeds" component={FeedsPage} />
          <Route path="/stash" component={StashPage} />
          <Route path="/feeds/:id" component={FeedPage} />
        </AuthLayout>
      </Route>
    </Switch>
  )
}
