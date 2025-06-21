import { Button, Separator } from '@colette/ui'
import { useConfig } from '@colette/util'

export const OidcButton = (props: { signInText: string }) => {
  const config = useConfig()

  return (
    <>
      <div className="flex items-center space-x-4">
        <Separator className="flex-1" />
        <span className="text-muted-foreground text-sm">Or continue with</span>
        <Separator className="flex-1" />
      </div>
      <Button asChild type="button" variant="outline">
        <a href={config.server.baseUrl + 'api/auth/oidc/redirect'}>
          {props.signInText ?? 'Sign in with OIDC'}
        </a>
      </Button>
    </>
  )
}
