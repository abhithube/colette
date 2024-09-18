import {
  type ApiClient,
  Login,
  Profile,
  Register,
  SwitchProfile,
  User,
} from './openapi.gen'

export interface AuthAPI {
  register(body: Register): Promise<User>

  login(body: Login): Promise<Profile>

  getActive(): Promise<User>

  switchProfile(data: SwitchProfile): Promise<Profile>
}

export class HTTPAuthAPI implements AuthAPI {
  constructor(private client: ApiClient) {}

  async register(data: Register): Promise<User> {
    return this.client
      .post('/auth/register', {
        body: await Register.parseAsync(data),
      })
      .then(User.parseAsync)
  }

  async login(data: Login): Promise<Profile> {
    return this.client
      .post('/auth/login', {
        body: await Login.parseAsync(data),
      })
      .then(Profile.parseAsync)
  }

  async getActive(): Promise<User> {
    return this.client.get('/auth/@me').then(User.parseAsync)
  }

  async switchProfile(data: SwitchProfile): Promise<Profile> {
    return this.client
      .post('/auth/switchProfile', {
        body: await SwitchProfile.parseAsync(data),
      })
      .then(Profile.parseAsync)
  }
}
