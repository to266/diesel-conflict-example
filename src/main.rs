use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel_derive_enum::DbEnum;

#[derive(diesel::sql_types::SqlType)]
#[diesel(postgres_type(name = "my_enum"))]
pub struct MyEnum;

#[derive(Debug, DbEnum)]
#[ExistingTypePath = "MyEnum"]
#[DbValueStyle = "verbatim"]
pub enum DbEnum {
    Normal,
    Special,
}

diesel::table! {
    use diesel::sql_types::*;
    use super::MyEnum;

    my_table (id) {
        id -> Int4,
        name -> Text,
        origin -> MyEnum,
        content -> Nullable<Text>,
    }
}

fn main() {
    let mut conn =
        PgConnection::establish("postgres://postgres:postgres@localhost:5432/postgres").unwrap();
    conn.batch_execute("
        drop table if exists my_table cascade;
        drop type if exists my_enum;
        create type my_enum as enum ('Normal', 'Special');
        create table my_table (
            id serial primary key,
            name text not null,
            origin my_enum not null,
            content text
        );
        create unique index if not exists my_table_name_origin_idx on my_table (name, origin) where origin <> 'Special';
        insert into my_table(name, origin, content) values ('test', 'Normal', 'One');
        ").unwrap();

    diesel::insert_into(my_table::table)
        .values((
            my_table::name.eq("no_conflict"),
            my_table::origin.eq(DbEnum::Normal),
            my_table::content.eq("two"),
        ))
        // Adding the commented lines below actually breaks
        // .on_conflict((my_table::name, my_table::origin))
        // .filter_target(my_table::origin.eq(DbEnum::Special))
        // .do_nothing()
        .execute(&mut conn)
        .expect("Counld not insert without any conflict");
    diesel::insert_into(my_table::table)
        .values((
            my_table::name.eq("test"),
            my_table::origin.eq(DbEnum::Normal),
            my_table::content.eq("two"),
        ))
        .on_conflict((my_table::name, my_table::origin))
        .filter_target(my_table::origin.eq(DbEnum::Special))
        .do_update()
        .set(my_table::content.eq(excluded(my_table::content)))
        .execute(&mut conn)
        .expect("Could not insert with conflict");
    todo!()
}
