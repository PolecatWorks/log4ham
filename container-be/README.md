# Tutorial for Getting Started in Rust

* Use Clap to create a CLI
* schema command
  * Use schemars to generate schema from Structures
  * Create a simple data class to represent the data using JsonSchema from schemars
  * output the schema with the schema_for macro
        large-json schema
* schema-list command
  * Use schemars to generate schema for Vec of our data class
        large-json schema-list
        cargo run -- schema-list
* generate command
  * Create a file and write n copies of the structure to the file
        large-json generate myfile.json --count 1000
* validate command
  * read a file and validate its json against the schema and count the number of records
        large-json validate myfile.json
* receive command
  * start a web service to allow upload of files and validation

      curl -v -d@myfile.json http://localhost:8080/test/v0/review

# Benchmarking
Benchmark some of our functions (eg validation)
Run with

    cargo bench

or

    cargo watch -x bench

Then view your benchmark results at ./target/criterion/list_sizes/report/index.html

# Docker

Build your docker image and then run it

    make docker-build

run in docker

    docker run -it localhost/polecatworks/large-json large-json receive

Tasks
* Include ValidationError encapsulated in MyError when rethrowing (But need to resolve lifetimes, etc)
* Genericise the Schema generation via a generic parameter on the scheme_string.
* Make an async benchmark to test the http
* Add soft shutdown based on signals
* Choose DB tooling
  * Non ORM
    * Consider: https://docs.rs/sqlx/latest/sqlx/
      Install a Schema Management
        https://crates.io/crates/sqlx-cli

        cargo install sqlx-cli
        sqlx migrate add -r initial
        sqlx migrate run

  * Check out the contents of postgres
      podman run --name some-postgres -p 5432:5432 -e POSTGRES_PASSWORD=mypw -d postgres

      psql -h localhost -p 5432 -U postgres

      create database mydb;

      create user myuser with encrypted password 'mypass';
      \dn
      \c mydb postgres
      create SCHEMA myuser AUTHORIZATION myuser;



      grant all privileges on database mydb to myuser;
      GRANT ALL ON SCHEMA public TO myuser;

      psql -h localhost -p 5432 -U myuser -d mydb
      export DATABASE_URL=postgres://myuser:mypass@localhost/mydb


      curl -v -d@myfile_good.json -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists


# Create Objects
Create the various data objects using the following curl commands

    curl -v -d@myfile_good.json -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists

    CREATE
    curl -X POST -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists -d '{"name":"myexample0"}'
    READ
    curl -X GET  http://localhost:8080/test/v0/lists
    curl -X GET  http://localhost:8080/test/v0/lists/1
    UPDATE
    curl -X PUT -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists/1 -d '{"name":"myexample0","id":1}'
    curl -X PUT -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists/1 -d '{"name":"myexample","id":1,"active":1}'
    DELETE
    curl -X DELETE http://localhost:8080/test/v0/lists/6



Create the version:
    CREATE
    curl -X POST -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists/1/versions -d '{"name":"myexample8","version":"1.2.3","schema":"{}","list":1}'
    curl -X POST -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists/1/versions  -d '{"name":"myexample8","version":"1.2.3","schema":"{}","list":1}'
    curl -X POST -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists/1/versions  -d@test_data/schema_versions.json

    curl -X GET  http://localhost:8080/test/v0/lists/1/versions


    READ
    curl -X GET http://localhost:8080/test/v0/lists/4/versions
    curl -X GET http://localhost:8080/test/v0/lists/4/versions/4

Create the file

    curl -X POST -H 'Content-Type: application/json' http://localhost:8080/test/v0/lists/1/files -d@myfile.json


# For actual DB use

  export DATABASE_URL=postgres://myuser:mypass@localhost/mydb

# for tests (superuser access)

Tests will dynamically create the DB and apply the migrations to the DB allowing clean testing.
Therefore tests need a superuser access to the DB

  export DATABASE_URL=postgres://postgres:mypw@localhost/postgres


# Build up sample data

Create some SQL that directly generates test data

INSERT INTO lists (name) VALUES ('example0') RETURNING *;
INSERT INTO lists (name) VALUES ('example1') RETURNING *;
INSERT INTO lists (name) VALUES ('example2') RETURNING *;

INSERT INTO list_versions (version,schema,list) VALUES ('0.0.1', '{}', 1) RETURNING *;
INSERT INTO list_versions (version,schema,list) VALUES ('0.0.2', '{}', 1) RETURNING *;
INSERT INTO list_versions (version,schema,list) VALUES ('0.0.1', '{}', 1) RETURNING *;

UPDATE lists SET active = 1 WHERE id=1 RETURNING *;

INSERT INTO list_files (version, validated) SELECT active,false FROM lists WHERE id=1 RETURNING *;
