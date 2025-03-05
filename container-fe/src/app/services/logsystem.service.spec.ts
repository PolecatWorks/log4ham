import { TestBed } from '@angular/core/testing';

import { LogsystemService } from './logsystem.service';

describe('LogsystemService', () => {
  let service: LogsystemService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(LogsystemService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
