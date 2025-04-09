import { DataSource } from '@angular/cdk/collections';
import { HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';

export interface SimpleDataSource<T> extends DataSource<T> {
  connect(): Observable<T[]>;
  disconnect(): void;
}

export interface Sort<T> {
  property: keyof T;
  order: 'asc' | 'desc';
}

export interface PageOptions<T> {
  page?: number;
  size: number;
  sort?: Sort<T>;
}

export function asHttpParams<T>(options: PageOptions<T>): HttpParams {
  const simpleObj: Record<string, string> = {};

  simpleObj['size'] = String(options.size);

  if (options.page != undefined) {
    simpleObj['page'] = String(options.page);
  }
  if (options.sort != undefined) {
    simpleObj['sortProperty'] = String(options.sort.property);
    simpleObj['sortOrder'] = options.sort.order;
  }

  return new HttpParams({ fromObject: simpleObj });
}

/**
 * Represents a paginated response.
 *
 * @template C - The type of the objects returned
 * @template T - The type of the options used to describe the page request.
 *
 * @property {C[]} ids - An array of identifiers for the current page.
 * @property {PageRequest<T>} options - The options for the current page request.
 */
export interface ListPages<C, T> {
  ids: C[];
  pagination: PageOptions<T>;
}

export type PaginatedEndpoint<C, T> = (req: PageOptions<T>) => Observable<ListPages<C, T>>;
export type SourceUpdate = () => Observable<number>;
