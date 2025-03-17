import { Component } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { Log4HamService } from '../../services/log4ham.service';
import { switchMap } from 'rxjs';
import { User } from '../../services/user';
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
  id: number | null = null;
  user: User = {} as User;

  constructor(
    private route: ActivatedRoute,
    private log4hamService: Log4HamService,
    private router: Router
  ) {
    this.route.params
      .pipe(
        switchMap(param => {
          return this.log4hamService.usersGet(param['id']);
        })
      )
      .subscribe(params => {
        this.user = params;
      });
  }

  submit() {
    this.log4hamService.usersUpdate(this.user).subscribe({
      next: data => {
        console.log('updated: ', data);
        this.router.navigate(['..'], { relativeTo: this.route });
      },
      error: error => {
        console.error('Error:', error);
      },
    });
  }
}
