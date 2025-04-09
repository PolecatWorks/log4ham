import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { User } from '../structs/user';
import { CollectionViewer, DataSource } from '@angular/cdk/collections';
import { Log4HamService } from './log4ham.service';

@Injectable({
  providedIn: 'root',
})
export class UsersDataSource extends DataSource<User> {
  users = new BehaviorSubject<User[]>([]);
  isLoading = new BehaviorSubject<boolean>(false);

  constructor(private log4hamService: Log4HamService) {
    super();
  }

  override connect(): Observable<User[]> {
    return this.users.asObservable();
  }

  override disconnect(collectionViewer: CollectionViewer): void {
    this.users.complete();
  }
}
