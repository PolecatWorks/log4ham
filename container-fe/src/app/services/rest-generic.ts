import { HttpClient, HttpParams } from "@angular/common/http";
import { catchError, forkJoin, map, Observable, Subject, switchMap, throwError } from "rxjs";
import { ListPages, PageOptions } from "./pagination";

export function asHttpParams<T>(options: PageOptions<T>): HttpParams {

  var simpleObj: { [id: string] : string; } = {};

  simpleObj['size'] = String(options.size);

  if (options.page != undefined) {
    simpleObj['page'] = String(options.page);
  }
  if (options.sort != undefined) {
    simpleObj["sortProperty"] = String(options.sort.property);
    simpleObj["sortOrder"] =  options.sort.order;
  }

  return new HttpParams({fromObject: simpleObj});
}



export class RestGeneric<T> {
  constructor(
    private http: HttpClient,
    public url: string,
    public name: string,
  ) {}

  private source = new Subject<number>();

  sourceUpdate() {
    return this.source.asObservable();
  }

  sourceRefresh(now: number) {
    this.source.next(now);
  }

  getPagedIds(query: PageOptions<T>) {
    const params = asHttpParams(query);

    return this.http.get<ListPages<number, T>>(this.url, { params: params }).pipe(
      catchError(error => {
        console.error('Error:', error);
        return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
      })
    );
  }

  get(id: number) {
    return this.http.get<T>(this.url + '/' + id).pipe(
      catchError(error => {
        console.error('Error:', error);
        return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
      })
    );
  }
  // Get paged detail providing the detail of each page in a full list
  getPagedDetail(query: PageOptions<T>): Observable<ListPages<T, T>> {
    return this.getPagedIds(query).pipe(
      switchMap(idsPage => {
        const detailRequests = idsPage.ids.map(id => this.get(Number(id)));
        return forkJoin(detailRequests).pipe(
          map(details => ({
            ids: details,
            pagination: idsPage.pagination,
          }))
        );
      })
    );
  }


}
