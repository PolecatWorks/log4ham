import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root',
})
export class AuthService {
  private userId: number | null = null;

  constructor(private http: HttpClient) {
    console.log('AuthService initialized');
    this.userId = 1;
  }

  getUserId(): number {
    if (this.userId === null) {
      console.warn('User ID is not set');
      // TODO: Handle the case where userId is not set and remove this from production code
      return -1;
    }
    return this.userId;
  }
}
