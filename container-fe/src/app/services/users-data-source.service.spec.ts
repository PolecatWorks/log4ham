import { TestBed } from '@angular/core/testing';

import { UsersDataSource } from './users-data-source.service';

describe('UsersDataSourceService', () => {
  let service: UsersDataSource;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(UsersDataSource);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
