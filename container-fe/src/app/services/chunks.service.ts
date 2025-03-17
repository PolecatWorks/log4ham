import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { IChunk } from '../components/chunks/ichunk';
import { catchError } from 'rxjs/operators';
import { throwError } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class ChunksService {
  constructor(private http: HttpClient) {}

  sendChunks(name: string, chunkName: string, size: number) {
    return this.http.post('/' + name + '/v0' + '/chunks', { name: chunkName, num_chunks: size }).pipe(
      catchError(error => {
        console.error('Error:', error);
        return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
      })
    );
  }

  getChunks(name: string) {
    return this.http.get<IChunk>('/' + name + '/v0' + '/chunks').pipe(
      catchError(error => {
        console.error('Error:', error);
        return throwError(() => new Error('Could not process request: ' + error.message + ' (Status code: ' + error.status + ')'));
      })
    );
  }
}
