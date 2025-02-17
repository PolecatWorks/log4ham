import * as fs from 'fs';

const testdatadir = './testdata';

// const fs = require('fs')

var defaultTarget = "http://dev.k8s/";

const sampleChunk = {
  "name": "chunks1",
  "size": 100,
}

const sampleBuckets = [
  {
    "location": "abcdef.json",
    last_modified: "2019-01-01T00:00:00Z",
    size: 100,
    e_tag: "1234567890abcdef",
    version: null,
  },
  {
    "location": "ffff",
    last_modified: "2019-01-01T00:00:00Z",
    size: 100000,
    e_tag: "1234567890cdef",
    version: null,
  }
]

console.log("Setting up bypass");
var PREFIX = "/pie/v0/chunks";



export default {
  "/home/**": {
    target: defaultTarget,
    secure: false,
    logLevel: "debug",
    bypass: function (req, res, proxyOptions) {
      let url = req.url;
      console.log("Bypassing for", url);
      return "/index.html";
    },
  },

  "/auth": {
    target: "http://dev.k8s/",
    secure: false,
    logLevel: "debug",
    changeOrigin: true,
  },

  // "/pie/v0/chunks": {
  //   target: "http://dev.k8s/",
  //   secure: false,
  //   logLevel: "debug",
  //   changeOrigin: true,
  // },

  "/pie/v0/chunks": {
    target: defaultTarget,
    secure: false,
    logLevel: "debug",
    changeOrigin: true,

    configure: (proxy, _options) => {
      proxy.on('error', (err, req, res) => {

        let url = req.url;

        if (url.startsWith(PREFIX)) {
          console.log("Proxy failed for ",url, " so substituting with test data")
          // PREFIX is exactly at the beginning
          url = url.slice(PREFIX.length);
          if (url === "") {
            res.end(JSON.stringify(sampleChunk));
          } else {
            try {
              const fileContents = fs.readFileSync(testdatadir+url, { encoding: 'utf8'});
              res.end(fileContents);
            } catch (err) {
            res.writeHead(404, {
              'Content-Type': 'text/plain',
            });

            res.end(JSON.stringify({'error': 'File not found', 'file': testdatadir+url}));
            }

          }

          return;
        }
        console.log('proxy error', err);
        // res.writeHead(500, {
        //   'Content-Type': 'text/plain',
        // });
        // res.end('Something went wrong. And we are reporting a custom error message.' + err);
      });
      proxy.on('proxyReq', (proxyReq, req, _res) => {
        console.log('Sending Request to the Target:', req.method, req.url);
      });
      proxy.on('proxyRes', (proxyRes, req, _res) => {
        console.log('Received Response from the Target:', proxyRes.statusCode, req.url);
      });
    },

  },
  // {
  //   context: [
  //     "**"
  //   ],
  //   target: defaultTarget,
  //   secure: false,
  //   logLevel: "debug",
  //   bypass: function (req, res, proxyOptions) {
  //     let url = req.url;
  //     // console.log("Returning local file for", url, "with", res);
  //     console.log("GOT ANOTHER BYPASS", url);
  //     return null;
  //   },
  // },
};
