-- Your SQL goes here
create table if not exists school_classes (
    id serial primary key not null,
    school_id int not null,
    name text not null,
    boys int not null,
    girls int not null
);