-- Your SQL goes here
CREATE TABLE IF NOT EXISTS schools (
    id serial PRIMARY KEY NOT NULL,
    name text NOT NULL,
    email text not null,
    password text not null
);