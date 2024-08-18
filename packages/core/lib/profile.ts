import {
  type ApiClient,
  Profile,
  ProfileCreate,
  ProfileList,
  ProfileUpdate,
} from './openapi.gen'

export interface ProfileAPI {
  list(): Promise<ProfileList>

  get(id: string): Promise<Profile>

  getActive(): Promise<Profile>

  create(data: ProfileCreate): Promise<Profile>

  update(id: string, data: ProfileUpdate): Promise<Profile>

  delete(id: string): Promise<void>
}

export class HTTPProfileAPI implements ProfileAPI {
  constructor(private client: ApiClient) {}

  async list(): Promise<ProfileList> {
    return this.client.get('/profiles').then(ProfileList.parseAsync)
  }

  async get(id: string): Promise<Profile> {
    return this.client
      .get('/profiles/{id}', {
        path: {
          id,
        },
      })
      .then(Profile.parseAsync)
  }

  async getActive(): Promise<Profile> {
    return this.client.get('/profiles/@me').then(Profile.parseAsync)
  }

  async create(body: ProfileCreate): Promise<Profile> {
    return this.client
      .post('/profiles', {
        body: await ProfileCreate.parseAsync(body),
      })
      .then(Profile.parseAsync)
  }

  async update(id: string, body: ProfileUpdate): Promise<Profile> {
    return this.client
      .patch('/profiles/{id}', {
        path: {
          id,
        },
        body: await ProfileUpdate.parseAsync(body),
      })
      .then(Profile.parseAsync)
  }

  async delete(id: string): Promise<void> {
    return this.client
      .delete('/profiles/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
