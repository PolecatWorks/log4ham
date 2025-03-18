import { Inject, Injectable } from '@angular/core';
import { BehaviorSubject, combineLatest, map, Observable, Subject, switchMap } from 'rxjs';
import { ListPages, PaginatedEndpoint, SimpleDataSource, Sort, SourceUpdate } from './pagination';

@Injectable({
  providedIn: 'root',
})
export class PaginationDataSource<T> implements SimpleDataSource<T> {
  private sort = new Subject<Sort<T>>();
  private pageNumber = new Subject<number>();

  public page: Observable<ListPages<T, T>>;

  datas = new BehaviorSubject<T[]>([]);
  sourceUpdate: Observable<number>;

  constructor(
    @Inject('PaginatedEndpoint') endpoint: PaginatedEndpoint<T, T>,
    @Inject('SourceUpdate') sourceUpdate: Observable<number>,
    @Inject('InitialSort') initialSort: Sort<T>,
    @Inject('InitialPage') initialPage: number
  ) {
    this.sourceUpdate = sourceUpdate;
    // this.sortBy(initialSort);
    // this.fetch(initialPage);

    this.page = combineLatest({
      sourceUpdate: sourceUpdate,
      sort: this.sort,
      pageNumber: this.pageNumber,
    }).pipe(
      switchMap(({ sourceUpdate, sort, pageNumber }) => {
        console.log('fetching page', pageNumber, 'with sort', sort, 'on', sourceUpdate);
        return endpoint({ page: pageNumber, sort: sort, size: 10 });
      })
    );
  }

  sortBy(sort: Sort<T>): void {
    this.sort.next(sort);
  }

  fetch(page: number): void {
    this.pageNumber.next(page);
    console.log('Made a fetch of ', page);
  }

  connect(): Observable<T[]> {
    console.log('PaginationDataSource.connect');

    return this.page.pipe(
      map(page => {
        const retval = page.ids;
        console.log('page', retval);
        return retval;
      })
    );
  }

  disconnect(): void {
    this.sort.complete();
    this.pageNumber.complete();
  }
}
