import { Component, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { ChunksService } from '../../services/chunks.service';
import { IChunk } from './ichunk';

@Component({
  selector: 'app-chunks',
  imports: [],
  templateUrl: './chunks.component.html',
  styleUrl: './chunks.component.scss',
})
export class ChunksComponent implements OnInit {
  constructor(
    private route: ActivatedRoute,
    private chunksApi: ChunksService
  ) {}

  chunks = 0;
  chunkName = '';

  ngOnInit(): void {
    this.route.params.subscribe(params => {
      this.chunkName = params['name'];
      this.getChunks(this.chunkName);
    });
  }

  getChunks(id: string) {
    this.chunksApi.getChunks(id).subscribe({
      next: (data: IChunk) => {
        this.chunks = data.chunks;
      },
      error: error => {
        console.error('Error:', error);
        this.chunks = -1;
        this.chunkName = id + ': Error (' + error.message + ')';
      },
    });
  }

  sendChunks() {
    this.chunksApi.sendChunks(this.chunkName, 'ABC', 20).subscribe({
      next: data => {
        console.log(data);
      },
      error: error => {
        console.error('Error:', error);
        this.chunks = -1;
        this.chunkName = this.chunkName + ': Error (' + error.message + ')';
      },
    });
    this.getChunks(this.chunkName);
  }
}
