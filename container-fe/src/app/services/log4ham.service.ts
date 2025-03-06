import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { catchError, forkJoin, map, switchMap, throwError } from 'rxjs';
import { ListIds } from './list-options';
import { User } from './user';

@Injectable({
  providedIn: 'root'
})
export class Log4HamService {

  constructor(private http: HttpClient) { }
  private prefix = '/log4ham';


  getLogIds() {
    return this.http.get(this.prefix + '/logs')
      .pipe(
        catchError((error: any) => {
          console.error('Error:', error);
          return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
        })
      )
  }

  getUserIds() {
    return this.http.get<ListIds>(this.prefix + '/users')
      .pipe(
        catchError((error: any) => {
          console.error('Error:', error);
          return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
        })
      )
  }

  getUserDetails() {
    return this.getUserIds()
      .pipe(
        switchMap((ids) => {
          const detailRequests = ids.ids.map(id => this.usersGet(Number(id)));
          return forkJoin(detailRequests);
        }),
        map(details => details.flat())
    )
  }

  usersGet(id: Number) {
    return this.http.get<User>(this.prefix + '/users/' + id)
      .pipe(
        catchError((error: any) => {
          console.error('Error:', error);
          return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
        })
      )
  }

  usersCreate(forename: string, surname: string, password: string) {
    return this.http.post(this.prefix + '/users', { forename: forename, surname: surname, password: password })
      .pipe(
        catchError((error: any) => {
          console.error('Error:', error);
          return throwError(() => new Error('Could not create new user: ' + error.message + ' (Status code: ' + error.status + ')'));
        })
      )
  }
}
