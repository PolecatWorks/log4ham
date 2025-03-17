import { HttpInterceptorFn, HttpRequest } from '@angular/common/http';
import { inject } from '@angular/core';
import { OAuthService } from 'angular-oauth2-oidc';

export const authInterceptor: HttpInterceptorFn = (req: HttpRequest<unknown>, next) => {
  const oauthService = inject(OAuthService);
  const accessToken = oauthService.getAccessToken();

  if (accessToken) {
    const cloned = req.clone({
      headers: req.headers.set('Authorization', `Bearer ${accessToken}`),
    });
    return next(cloned);
  } else {
    return next(req);
  }
};
