import { Routes } from '@angular/router';
import { HomeComponent } from './components/home/home.component';
import { ChunksComponent } from './components/chunks/chunks.component';
import { LoginUserComponent } from './components/login-user/login-user.component';
import { UsersComponent } from './components/users/users.component';
import { LogsComponent } from './components/logs/logs.component';
import { UserComponent } from './components/user/user.component';

export const routes: Routes = [
  {
    path: 'home',
    component: HomeComponent,
    children: [
      {
        path: 'chunks/:name',
        component: ChunksComponent,
      },
      {
        path: 'user',
        component: LoginUserComponent,
      },
      {
        path: 'users',
        component: UsersComponent,
        children: [
          {
            path: ':id',
            component: UserComponent,
          },
        ],
      },
      {
        path: 'logs',
        component: LogsComponent,
      },
    ],
  },
  { path: '**', pathMatch: 'full', redirectTo: 'home' },
];
