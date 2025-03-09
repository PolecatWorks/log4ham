import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { catchError, forkJoin, map, Observable, switchMap, throwError } from 'rxjs';
import { User } from './user';
import { ListPages, PageOptions } from './pagination';

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


  usersGetIdsPaged(query: PageOptions<User>) {
    const params = new HttpParams({ fromObject: query as any });

    return this.http.get<ListPages<number, User>>(this.prefix + '/users', { params: params })
      .pipe(
      catchError((error: any) => {
        console.error('Error:', error);
        return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
      })
      )
  }

  usersGetDetailPaged(query: PageOptions<User>): Observable<ListPages<User, User>> {
    return this.usersGetIdsPaged(query)
      .pipe(
        switchMap((idsPage) => {
          const detailRequests = idsPage.ids.map(id => this.usersGet(Number(id)));
          return forkJoin(detailRequests).pipe(
            map(details => ({
              ids: details,
              pagination: idsPage.pagination,
            }))
          );
        })
    )
  }

  usersGetIds() {
    return this.http.get<ListPages<number, User>>(this.prefix + '/users')
      .pipe(
        catchError((error: any) => {
          console.error('Error:', error);
          return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
        })
      )
  }

  usersGetDetail() {
    return this.usersGetIds()
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
