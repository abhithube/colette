import { Route, Router } from '@solidjs/router'
import type { Component } from 'solid-js'
import { AuthLayout } from './auth-layout'
import { FeedPage } from './feeds/id'
import { LoginPage } from './login'

const App: Component = () => {
  return (
    <Router>
      <Route path="" component={AuthLayout}>
        <Route path="/" component={() => <div>Home</div>} />
        <Route path="/feeds/:id" component={FeedPage} />
      </Route>
      <Route path="/login" component={LoginPage} />
    </Router>
  )
}

export default App
