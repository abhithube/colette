import {
  type AuthAPI,
  Login,
  Profile,
  Register,
  SwitchProfile,
  User,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class AuthCommands implements AuthAPI {
  register(data: Register): Promise<User> {
    return invoke('register', { data: Register.parse(data) }).then(User.parse)
  }

  login(data: Login): Promise<Profile> {
    return invoke('login', { data: Login.parse(data) }).then(Profile.parse)
  }

  getActive(): Promise<User> {
    return invoke('get_active_user').then(User.parse)
  }

  switchProfile(data: SwitchProfile): Promise<Profile> {
    return invoke('switch_profile', { data: SwitchProfile.parse(data) }).then(
      Profile.parse,
    )
  }
}
