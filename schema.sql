drop table if EXISTS todos;

create table todos (
	id serial primary key,
	note text not null
);
