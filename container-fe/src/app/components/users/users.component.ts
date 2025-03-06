import { Component, ViewChild } from '@angular/core';
import { Log4HamService } from '../../services/log4ham.service';
import { CommonModule } from '@angular/common';
import { forkJoin } from 'rxjs/internal/observable/forkJoin';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { switchMap, map } from 'rxjs/operators';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { User } from '../../services/user';

const ELEMENT_DATA: User[] = [ { "id": 1, "forename": "Sharon", "surname": "Greene", "password": "abc" }, { "id": 2, "forename": "Ben", "surname": "Greene", "password": "abc" }, { "id": 3, "forename": "Sam", "surname": "Greene", "password": "abc" } ];


@Component({
  imports: [CommonModule, MatTableModule, MatPaginatorModule],
  templateUrl: './users.component.html',
  styleUrl: './users.component.scss'
})
export class UsersComponent {

  constructor(private logsystemApi: Log4HamService) { }

  dataSource = new MatTableDataSource<User>(ELEMENT_DATA);

  @ViewChild(MatPaginator) paginator!: MatPaginator;

  ngAfterViewInit() {
    this.dataSource.paginator = this.paginator;
  }

  displayedColumns: string[] = ['forename', 'surname'];


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
