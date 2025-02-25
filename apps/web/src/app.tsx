import { AuthLayout } from './auth-layout'
import { StashPage } from './routes/bookmarks'
import { FeedsPage } from './routes/feeds'
import { FeedPage } from './routes/feeds/id'
import { HomePage } from './routes/home'
import { LoginPage } from './routes/login'
import { RegisterPage } from './routes/register'
import type { FC } from 'react'
import { Route, Switch } from 'wouter'

export const App: FC = () => {
  return (
    <Switch>
      <Route path="/register" component={RegisterPage} />
      <Route path="/login" component={LoginPage} />
      <Route path="/" nest>
        <AuthLayout>
          <Route path="/" component={HomePage} />
          <Route path="/feeds" component={FeedsPage} />
          <Route path="/stash" component={StashPage} />
          <Route path="/feeds/:id" component={FeedPage} />
        </AuthLayout>
      </Route>
    </Switch>
  )
}
