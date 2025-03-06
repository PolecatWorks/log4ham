import { TestBed } from '@angular/core/testing';

import { UsersDataSourceService } from './users-data-source.service';

describe('UsersDataSourceService', () => {
  let service: UsersDataSourceService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(UsersDataSourceService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
