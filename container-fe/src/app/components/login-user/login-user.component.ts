import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { OAuthService } from 'angular-oauth2-oidc';
import { jwtDecode } from 'jwt-decode';

@Component({
  selector: 'login-user',
  imports: [CommonModule],
  templateUrl: './login-user.component.html',
  styleUrl: './login-user.component.scss',
})
export class LoginUserComponent {
  constructor(private oauthService: OAuthService) {}

  get userName(): string {
    const claims = this.oauthService.getIdentityClaims();
    if (!claims) return 'null';
    return claims['given_name'];
  }

  get resource_access() {
    const accessToken = this.oauthService.getAccessToken();
    if (!accessToken) return null;

    const decodedToken: any = jwtDecode(accessToken);
    // console.log(decodedToken);
    return decodedToken.resource_access || null;
  }

  get idToken(): string {
    this.oauthService;
    return this.oauthService.getIdToken();
  }

  get accessToken(): string {
    return this.oauthService.getAccessToken();
  }

  refresh() {
    this.oauthService.refreshToken();
  }

  logout() {
    this.oauthService.logOut();
  }
}
