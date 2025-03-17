import { Inject, Injectable } from '@angular/core';
import { BehaviorSubject, combineLatest, map, Observable, Subject, switchMap } from 'rxjs';
import { ListPages, PaginatedEndpoint, SimpleDataSource, Sort } from './pagination';

@Injectable({
  providedIn: 'root',
})
export class PaginationDataSource<T> implements SimpleDataSource<T> {
  private sort = new Subject<Sort<T>>();
  private pageNumber = new Subject<number>();

  public page: Observable<ListPages<T, T>>;

  datas = new BehaviorSubject<T[]>([]);

  constructor(
    @Inject('PaginatedEndpoint') endpoint: PaginatedEndpoint<T, T>,
    @Inject('InitialSort') initialSort: Sort<T>,
    @Inject('InitialPage') initialPage: number
  ) {
    this.pageNumber.subscribe({
      next: data => {
        console.log('pageNumber.next =', data);
      },
      error: error => {
        console.log('pageNumber.error:', error);
      },
      complete: () => {
        console.log('pageNumber.complete');
      },
    });

    this.sort.subscribe({
      next: data => {
        console.log('sort.next =', data);
      },
      error: error => {
        console.log('sort.error:', error);
      },
      complete: () => {
        console.log('sort.complete');
      },
    });

    this.page = combineLatest({
      sort: this.sort,
      pageNumber: this.pageNumber,
    }).pipe(
      switchMap(({ sort, pageNumber }) => {
        console.log('fetching page', pageNumber, 'with sort', sort);
        return endpoint({ page: pageNumber, sort: sort, size: 10 });
      })
    );

    // this.page.subscribe({
    //   next: (data) => {
    //     console.log("page.next =", data)
    //   },
    //   error: (error) => {
    //     console.log('page.error:', error);
    //   },
    //   complete: () => {
    //     console.log("page.complete")
    //   }
    // });

    // this.pageNumber.next(initialPage);
    // this.sort.next(initialSort);

    // this.page.subscribe(
    //   page => {
    //     console.log("page", page);
    //   }
    // );
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
