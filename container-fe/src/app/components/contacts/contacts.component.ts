import { Component, ViewChild, AfterViewInit } from '@angular/core';
import { Log4HamService } from '../../services/log4ham.service';
import { Contact } from '../../structs/contact';
import { PaginationDataSource } from '../../services/paginated-data-source.service';
import { PageOptions } from '../../services/pagination';
import { CommonModule } from '@angular/common';
import { MatTableModule } from '@angular/material/table';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { RouterLink, RouterOutlet } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';

@Component({
  selector: 'app-contacts',
  imports: [CommonModule, MatTableModule, MatPaginatorModule, RouterOutlet, RouterLink, MatButtonModule, MatIconModule],
  templateUrl: './contacts.component.html',
  styleUrl: './contacts.component.scss',
})
export class ContactsComponent implements AfterViewInit {
  displayedColumns = ['callsign', 'qso_date', 'qso_time', 'frequency', 'band', 'mode'];
  data: PaginationDataSource<Contact>;

  constructor(private log4Ham: Log4HamService) {
    this.data = new PaginationDataSource<Contact>(
      (request: PageOptions<Contact>) => this.log4Ham.contact.getPagedDetail(request),
      this.log4Ham.contact.sourceUpdate(),
      { property: 'id', order: 'asc' },
      1
    );

    this.log4Ham.contact.getPagedIds({ page: 0, size: 10 }).subscribe(data => {
      console.log('Contact IDs via Generic:', data);
    });
  }

  ngAfterViewInit(): void {
    this.data.sortBy({ property: 'id', order: 'asc' });
    this.data.fetch(1);
    this.log4Ham.contact.sourceRefresh(Date.now());
    console.log('Have send sortBy and fetch');
  }
  @ViewChild(MatPaginator) paginator!: MatPaginator;
}
