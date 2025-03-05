import { Component } from '@angular/core';
import { LogsystemService } from '../../services/logsystem.service';
import { CommonModule } from '@angular/common';
import { forkJoin } from 'rxjs/internal/observable/forkJoin';
import { switchMap, map } from 'rxjs/operators';

@Component({
  selector: 'app-users',
  imports: [CommonModule],
  templateUrl: './users.component.html',
  styleUrl: './users.component.scss'
})
export class UsersComponent {

  constructor(private logsystemApi: LogsystemService) { }

  userIds = {};
  logIds = {};

  userDetails = {};

  ngOnInit(): void {
    this.getUserIds();
    this.getLogIds();
    this.usersGetDetail();
  }

  getUserIds() {
    this.logsystemApi.getUserIds()
      .subscribe({
        next: (data) => {
          this.userIds = data;
        },
        error: (error) => {
          console.error('Error:', error);
          this.userIds = -1;
        }
      });
  }

  usersGetDetail() {
    this.logsystemApi.getUserIds()
    .pipe(
      switchMap((ids) => {
        const detailRequests = ids.ids.map(id => this.logsystemApi.usersGet(Number(id)));
        return forkJoin(detailRequests);
      }),
      map(details => details.flat())
    )
    .subscribe({
      next: (data) => {
        this.userDetails = data;
      },
      error: (error) => {
        console.error('Error:', error);
        this.userDetails = [];
      }
    });
  }

  getLogIds() {
    this.logsystemApi.getLogIds()
      .subscribe({
        next: (data) => {
          this.logIds = data;
        },
        error: (error) => {
          console.error('Error:', error);
          this.logIds = -1;
        }
      });
  }

  usersCreate(forename: string, surname: string, password: string) {
    this.logsystemApi.usersCreate(forename, surname, password)
      .subscribe({
        next: (data) => {
          console.log(data);
          this.getUserIds();
        },
        error: (error) => {
          console.error('Error:', error);
          this.userIds = -1;
        }
      });
  }


}
