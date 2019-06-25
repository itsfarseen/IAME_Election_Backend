-- Your SQL goes here
create table if not exists votes(
    id serial primary key not null,
    candidate_id int not null
);