import { Routes } from '@angular/router';
import { HomeComponent } from './components/home/home.component';
import { ChunksComponent } from './components/chunks/chunks.component';
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
                component: UserComponent,
            },
        ]
    },
    { path: '**', pathMatch: 'full', redirectTo: 'home'},
];
