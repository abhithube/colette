import { Route, Router } from '@solidjs/router'
import { render } from 'solid-js/web'
import './index.css'
import { Home } from './routes/home'
import { Login } from './routes/login'
import { Register } from './routes/register'

render(
	() => (
		<Router>
			<Route path="/login" component={Login} />
			<Route path="/register" component={Register} />
			<Route path="/" component={Home} />
		</Router>
	),
	document.getElementById('root')!,
)
