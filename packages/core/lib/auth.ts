import { type ApiClient, Login, Register, User } from './openapi.gen'

export interface AuthAPI {
  register(data: Register): Promise<User>

  login(data: Login): Promise<User>

  getActive(): Promise<User>
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

  login(data: Login): Promise<User> {
    return this.client
      .post('/auth/login', {
        body: Login.parse(data),
      })
      .then(User.parse)
  }

  getActive(): Promise<User> {
    return this.client.get('/auth/@me').then(User.parse)
  }
}
