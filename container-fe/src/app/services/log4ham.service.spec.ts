import { TestBed } from '@angular/core/testing';

import { Log4HamService } from './log4ham.service';

describe('Log4HamService', () => {
  let service: Log4HamService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(Log4HamService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
