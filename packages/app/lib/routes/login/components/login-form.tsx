import { Alert, Card, Button } from '@colette/ui'
import { useOIDCConfig } from '@colette/util'
import { UserCheck } from 'lucide-react'
import * as client from 'openid-client'

export const LoginForm = (props: { loggedOut?: boolean }) => {
  const oidcConfig = useOIDCConfig()

  async function handleClick() {
    const codeVerifier = client.randomPKCECodeVerifier()
    const codeChallenge = await client.calculatePKCECodeChallenge(codeVerifier)
    sessionStorage.setItem('colette-code-verifier', codeVerifier)

    const url = client.buildAuthorizationUrl(oidcConfig.clientConfig, {
      redirect_uri: oidcConfig.redirectUri,
      scope: 'openid,email,profile',
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
    })

    window.location.href = url.href
  }

  return (
    <>
      {props.loggedOut && (
        <Alert.Root className="mb-4">
          <UserCheck />
          <Alert.Title>Logged out</Alert.Title>
          <Alert.Description>
            You have been logged out of your account.
          </Alert.Description>
        </Alert.Root>
      )}
      <Card.Root>
        <Card.Header>
          <Card.Title>Login</Card.Title>
          <Card.Description>Login to your account</Card.Description>
        </Card.Header>
        <Card.Content>
          <Button onClick={handleClick}>Redirect</Button>
        </Card.Content>
      </Card.Root>
    </>
  )
}
