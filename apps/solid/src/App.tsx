import type { Component } from 'solid-js'
import { LoginForm } from './login/login-form'

const App: Component = () => {
  return (
    <div class="flex justify-center items-center h-screen">
      <div class="w-[400px]">
        <LoginForm />
      </div>
    </div>
  )
}

export default App
