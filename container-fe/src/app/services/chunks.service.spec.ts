import { TestBed } from '@angular/core/testing';

import { ChunksService } from './chunks.service';

describe('ChunksService', () => {
  let service: ChunksService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ChunksService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
