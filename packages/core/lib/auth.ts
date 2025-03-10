import { type ApiClient, Login, Register, User } from './openapi.gen'

export interface AuthAPI {
  registerUser(data: Register): Promise<User>

  loginUser(data: Login): Promise<User>

  getActiveUser(): Promise<User>

  logoutUser(): Promise<void>
}

export class HTTPAuthAPI implements AuthAPI {
  constructor(private client: ApiClient) {}

  registerUser(data: Register): Promise<User> {
    return this.client
      .post('/auth/register', {
        body: Register.parse(data),
      })
      .then(User.parse)
  }

  loginUser(data: Login): Promise<User> {
    return this.client
      .post('/auth/login', {
        body: Login.parse(data),
      })
      .then(User.parse)
  }

  getActiveUser(): Promise<User> {
    return this.client.get('/auth/@me').then(User.parse)
  }

  logoutUser(): Promise<void> {
    return this.client.post('/auth/logout').then()
  }
}
