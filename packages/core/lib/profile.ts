import {
  type ApiClient,
  Paginated_Profile,
  Profile,
  ProfileCreate,
  ProfileUpdate,
} from './openapi.gen'

export type ProfileList = Paginated_Profile
export const ProfileList = Paginated_Profile

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

  list(): Promise<ProfileList> {
    return this.client.get('/profiles').then(ProfileList.parse)
  }

  get(id: string): Promise<Profile> {
    return this.client
      .get('/profiles/{id}', {
        path: {
          id,
        },
      })
      .then(Profile.parse)
  }

  getActive(): Promise<Profile> {
    return this.client.get('/profiles/@me').then(Profile.parse)
  }

  create(body: ProfileCreate): Promise<Profile> {
    return this.client
      .post('/profiles', {
        body: ProfileCreate.parse(body),
      })
      .then(Profile.parse)
  }

  update(id: string, body: ProfileUpdate): Promise<Profile> {
    return this.client
      .patch('/profiles/{id}', {
        path: {
          id,
        },
        body: ProfileUpdate.parse(body),
      })
      .then(Profile.parse)
  }

  delete(id: string): Promise<void> {
    return this.client
      .delete('/profiles/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
