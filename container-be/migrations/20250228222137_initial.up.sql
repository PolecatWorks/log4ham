-- Add up migration script here

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    forename VARCHAR ( 50 ) NOT NULL,
    surname VARCHAR ( 50 ) NOT NULL,
    password VARCHAR ( 50 ) NOT NULL
);

CREATE TABLE logs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users ON DELETE RESTRICT,
    description VARCHAR ( 1000 ) NOT NULL
);
