create table if not exists elections(
    id serial primary key not  null,
    school_id int not null,
    name text not null,
    presidential bool not null,
    genders int not null
);