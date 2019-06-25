-- Your SQL goes here
create table if not exists candidates(
    id serial primary key not null,
    name text not null,
    school_id int not null,
    election_id int not null,
    class_id int not null,
    gender int not null,
    symbol text not null
);