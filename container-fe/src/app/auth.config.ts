import { AuthConfig } from 'angular-oauth2-oidc';

export const authCodeFlowConfig: AuthConfig = {
  issuer: 'http://dev.k8s/auth/realms/dev',
  redirectUri: window.location.origin + '/index.html',
  clientId: 'app-ui',
  responseType: 'code',
  scope: 'openid profile email offline_access',
  useSilentRefresh: true,
  showDebugInformation: true,
  timeoutFactor: 0.01,
  requireHttps: false,
  skipIssuerCheck: true,
  strictDiscoveryDocumentValidation: false,
};
