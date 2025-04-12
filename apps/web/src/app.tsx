import { AuthLayout } from './auth-layout'
import { StashPage } from './routes/bookmarks'
import { CollectionPage } from './routes/collections/id'
import { HomePage } from './routes/home'
import { LoginPage } from './routes/login'
import { RegisterPage } from './routes/register'
import { StreamPage } from './routes/streams/id'
import { SubscriptionsPage } from './routes/subscriptions'
import { SubscriptionPage } from './routes/subscriptions/id'
import { Route, Switch } from 'wouter'

export const App = () => {
  return (
    <Switch>
      <Route path="/register" component={RegisterPage} />
      <Route path="/login" component={LoginPage} />
      <Route path="/" nest>
        <AuthLayout>
          <Route path="/" component={HomePage} />
          <Route path="/subscriptions" component={SubscriptionsPage} />
          <Route path="/stash" component={StashPage} />
          <Route path="/subscriptions/:id" component={SubscriptionPage} />
          <Route path="/streams/:id" component={StreamPage} />
          <Route path="/collections/:id" component={CollectionPage} />
        </AuthLayout>
      </Route>
    </Switch>
  )
}
