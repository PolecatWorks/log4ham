import { AfterViewInit, Component, ViewChild } from '@angular/core';
import { Log4HamService } from '../../services/log4ham.service';
import { CommonModule } from '@angular/common';
import { forkJoin } from 'rxjs/internal/observable/forkJoin';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { switchMap, map } from 'rxjs/operators';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { User } from '../../services/user';
import { UsersDataSource } from '../../services/users-data-source.service';
import { PaginationDataSource } from '../../services/paginated-data-source.service';
import { PageOptions, Sort } from '../../services/pagination';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';

const ELEMENT_DATA: User[] = [
  { id: 1, forename: 'Sharon', surname: 'Greene', password: 'abc' },
  { id: 2, forename: 'Ben', surname: 'Greene', password: 'abc' },
  { id: 3, forename: 'Sam', surname: 'Greene', password: 'abc' },
];

@Component({
  imports: [CommonModule, MatTableModule, MatPaginatorModule, RouterOutlet, RouterLink, MatButtonModule],
  templateUrl: './users.component.html',
  styleUrl: './users.component.scss',
})
export class UsersComponent implements AfterViewInit {
  displayedColumns: string[] = ['forename', 'surname'];

  constructor(private log4HamService: Log4HamService) {
    // console.log("fetch for dataSource");
    // this.data.fetch(1);
  }
  ngAfterViewInit(): void {
    this.data.sortBy({ property: 'surname', order: 'asc' });
    this.data.fetch(1);
    console.log('Have send sortBy and fetch');
    // throw new Error('Method not implemented.');
  }

  data = new PaginationDataSource<User>(
    (request: PageOptions<User>) => this.log4HamService.usersGetPagedDetail(request),
    { property: 'surname', order: 'asc' },
    1
  );

  @ViewChild(MatPaginator) paginator!: MatPaginator;

  clickRow(_t39: any) {
    throw new Error('Method not implemented.');
  }

  // getUserIds() {
  //   this.log4HamService.usersGetIds()
  //     .subscribe({
  //       next: (data) => {
  //         this.userIds = data;
  //       },
  //       error: (error) => {
  //         console.error('Error:', error);
  //         this.userIds = -1;
  //       }
  //     });
  // }

  // usersGetDetail() {
  //   this.log4HamService.usersGetDetail()
  //   .subscribe({
  //     next: (data) => {
  //       console.log(data);
  //       this.userDetails = data;
  //     },
  //     error: (error) => {
  //       console.error('Error:', error);
  //       this.userDetails = [];
  //     }
  //   });
  // }

  // getLogIds() {
  //   this.log4HamService.getLogIds()
  //     .subscribe({
  //       next: (data) => {
  //         this.logIds = data;
  //       },
  //       error: (error) => {
  //         console.error('Error:', error);
  //         this.logIds = -1;
  //       }
  //     });
  // }

  usersCreate(user: User) {
    this.log4HamService.usersCreate(user).subscribe({
      next: data => {
        console.log('create: ', data);
        this.data.fetch(0);
      },
      error: error => {
        console.error('Error:', error);
        // this.userIds = -1;
      },
    });
  }
}
