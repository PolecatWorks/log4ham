import { AfterViewInit, Component, ViewChild } from '@angular/core';
import { Log4HamService } from '../../services/log4ham.service';
import { CommonModule } from '@angular/common';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { User } from '../../services/user';
import { PaginationDataSource } from '../../services/paginated-data-source.service';
import { PageOptions } from '../../services/pagination';
import { RouterLink, RouterOutlet } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';

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
