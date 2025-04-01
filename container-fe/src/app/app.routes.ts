import { Routes } from '@angular/router';
import { HomeComponent } from './components/home/home.component';
import { ChunksComponent } from './components/chunks/chunks.component';
import { LoginUserComponent } from './components/login-user/login-user.component';
import { UsersComponent } from './components/users/users.component';
import { LogsComponent } from './components/logs/logs.component';
import { UserComponent } from './components/user/user.component';
import { LogComponent } from './components/log/log.component';
import { ContactsComponent } from './components/contacts/contacts.component';
import { ContactComponent } from './components/contact/contact.component';

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
            path: 'edit/:id',
            component: UserComponent,
          },
          {
            path: 'new',
            component: UserComponent,
          },
          // {path: '*', redirectTo: '/home/users'},
        ],
      },
      {
        path: 'logs',
        component: LogsComponent,
        children: [{ path: ':id', component: LogComponent }],
      },
      {
        path: 'contacts',
        component: ContactsComponent,
        children: [
          { path: 'new', component: ContactComponent },
          { path: ':id', component: ContactComponent },
        ],
      },
    ],
  },
  { path: '**', pathMatch: 'full', redirectTo: 'home' },
];
