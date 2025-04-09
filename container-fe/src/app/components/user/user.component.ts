import { Component } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { Log4HamService } from '../../services/log4ham.service';
import { of, switchMap } from 'rxjs';
import { User } from '../../structs/user';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { FormsModule } from '@angular/forms';

@Component({
  standalone: true,
  imports: [MatButtonModule, MatFormFieldModule, MatInputModule, FormsModule],
  templateUrl: './user.component.html',
  styleUrl: './user.component.scss',
})
export class UserComponent {
  user: User = {} as User;

  constructor(
    private activatedRoute: ActivatedRoute,
    private log4hamService: Log4HamService,
    private router: Router
  ) {
    this.activatedRoute.params
      .pipe(
        switchMap(param => {
          if ('id' in param) {
            return this.log4hamService.usersGet(param['id']);
          } else {
            console.log('id not provided so creating a new user');
            return of({} as User);
          }
        })
      )
      .subscribe(params => {
        this.user = params;
      });
  }

  private newRecord() {
    return this.user.id === undefined;
  }

  submit() {
    if (this.newRecord()) {
      this.log4hamService.usersCreate(this.user).subscribe({
        next: data => {
          console.log('Created: ', data);

          this.router.navigate(['..'], { relativeTo: this.activatedRoute });
        },
        error: error => {
          console.error('Error:', error);
        },
      });
    } else {
      this.log4hamService.usersUpdate(this.user).subscribe({
        next: data => {
          console.log('updated: ', data);

          this.router.navigate(['../..'], { relativeTo: this.activatedRoute });
        },
        error: error => {
          console.error('Error:', error);
        },
      });
    }
  }
  cancel() {
    if (this.newRecord()) {
      this.router.navigate(['..'], { relativeTo: this.activatedRoute });
    } else {
      this.router.navigate(['../..'], { relativeTo: this.activatedRoute });
    }
  }
}
