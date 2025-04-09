import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { Contact } from '../../structs/contact';
import { ActivatedRoute, Router } from '@angular/router';
import { Log4HamService } from '../../services/log4ham.service';
import { of, switchMap } from 'rxjs';
import { AuthService } from '../../services/auth.service';

@Component({
  selector: 'app-contact',
  imports: [MatButtonModule, MatFormFieldModule, MatInputModule, FormsModule, MatSlideToggleModule],
  templateUrl: './contact.component.html',
  styleUrl: './contact.component.scss',
})
export class ContactComponent {
  record: Contact = {} as Contact;

  constructor(
    private activatedRoute: ActivatedRoute,
    private log4hamService: Log4HamService,
    private authService: AuthService,
    private router: Router
  ) {
    this.activatedRoute.params
      .pipe(
        switchMap(param => {
          if ('id' in param) {
            return this.log4hamService.contact.get(param['id']);
          } else {
            console.log('id not provided so creating a new contact');

            const myNewRecord = {} as Contact;
            myNewRecord.user_id = authService.getUserId();
            const dateNow = new Date();
            myNewRecord.qso_date = new Date().toISOString().split('T')[0];
            myNewRecord.qso_time = `${dateNow.getHours()}:${dateNow.getMinutes()}`;
            myNewRecord.is_confirmed = false;
            console.log('myNewRecord: ', myNewRecord);
            return of(myNewRecord);
          }
        })
      )
      .subscribe(params => {
        this.record = params;
      });
  }

  private newRecord() {
    return this.record.id === undefined;
  }

  submit() {
    if (this.newRecord()) {
      this.log4hamService.contact.create(this.record).subscribe({
        next: data => {
          console.log('Created: ', data);

          console.log('typeof id is ', typeof data.id);

          this.router.navigate(['..'], { relativeTo: this.activatedRoute });
        },
        error: error => {
          console.error('Error:', error);
        },
      });
    } else {
      this.log4hamService.contact.update(this.record).subscribe({
        next: data => {
          console.log('updated: ', data);
          this.router.navigate(['..'], { relativeTo: this.activatedRoute });
        },
        error: error => {
          console.error('Error:', error);
        },
      });
    }
  }

  cancel() {
    this.router.navigate(['..'], { relativeTo: this.activatedRoute });
  }
}
