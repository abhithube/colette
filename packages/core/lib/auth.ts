import {
  type ApiClient,
  Login,
  Profile,
  Register,
  SwitchProfile,
  User,
} from './openapi.gen'

export interface AuthAPI {
  register(data: Register): Promise<User>

  login(data: Login): Promise<Profile>

  getActive(): Promise<User>

  switchProfile(data: SwitchProfile): Promise<Profile>
}

export class HTTPAuthAPI implements AuthAPI {
  constructor(private client: ApiClient) {}

  register(data: Register): Promise<User> {
    return this.client
      .post('/auth/register', {
        body: Register.parse(data),
      })
      .then(User.parse)
  }

  login(data: Login): Promise<Profile> {
    return this.client
      .post('/auth/login', {
        body: Login.parse(data),
      })
      .then(Profile.parse)
  }

  getActive(): Promise<User> {
    return this.client.get('/auth/@me').then(User.parse)
  }

  switchProfile(data: SwitchProfile): Promise<Profile> {
    return this.client
      .post('/auth/switchProfile', {
        body: SwitchProfile.parse(data),
      })
      .then(Profile.parse)
  }
}
