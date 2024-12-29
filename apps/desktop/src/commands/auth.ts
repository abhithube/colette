import { type AuthAPI, Login, Register, User } from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class AuthCommands implements AuthAPI {
  register(data: Register): Promise<User> {
    return invoke('register', { data: Register.parse(data) }).then(User.parse)
  }

  login(data: Login): Promise<User> {
    return invoke('login', { data: Login.parse(data) }).then(User.parse)
  }

  getActive(): Promise<User> {
    return invoke('get_active_user').then(User.parse)
  }

  async logout(): Promise<void> {}
}
