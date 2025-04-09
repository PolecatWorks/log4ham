import { CommonModule } from '@angular/common';
import { Component, ViewChild, AfterViewInit } from '@angular/core';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { MatTableModule } from '@angular/material/table';
import { PaginationDataSource } from '../../services/paginated-data-source.service';
import { PageOptions } from '../../services/pagination';
import { Log4HamService } from '../../services/log4ham.service';
import { Log } from '../../structs/log';
import { MatButtonModule } from '@angular/material/button';
import { RouterLink, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-logs',
  imports: [CommonModule, MatTableModule, MatPaginatorModule, MatButtonModule, RouterOutlet, RouterLink],
  templateUrl: './logs.component.html',
  styleUrl: './logs.component.scss',
})
export class LogsComponent implements AfterViewInit {
  displayedColumns: string[] = ['user', 'description'];
  data: PaginationDataSource<Log>;

  constructor(private log4HamService: Log4HamService) {
    // console.log("fetch for dataSource");
    // this.data.fetch(1);
    this.data = new PaginationDataSource<Log>(
      (request: PageOptions<Log>) => this.log4HamService.logsGetPagedDetail(request),
      this.log4HamService.logsSourceUpdate(),
      { property: 'user', order: 'asc' },
      1
    );
  }

  ngAfterViewInit(): void {
    this.data.sortBy({ property: 'user', order: 'asc' });
    this.data.fetch(1);
    console.log('Have send sortBy and fetch');
    // throw new Error('Method not implemented.');
  }

  @ViewChild(MatPaginator) paginator!: MatPaginator;

  logsCreate(log: Log) {
    this.log4HamService.logsCreate(log).subscribe({
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
