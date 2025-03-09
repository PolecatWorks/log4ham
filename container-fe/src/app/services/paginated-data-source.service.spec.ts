import { TestBed } from '@angular/core/testing';

import { PaginationDataSource } from './paginated-data-source.service';

describe('PaginatedDataSourceService', () => {
  let service: PaginationDataSource<number>;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(PaginationDataSource);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
