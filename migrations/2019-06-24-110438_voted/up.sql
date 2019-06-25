-- Your SQL goes here
create table if not exists voted(
    id serial primary key not null,
    voter_num int not null,
    class_id int not null
)