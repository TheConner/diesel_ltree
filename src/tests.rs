extern crate dotenv;

use diesel::select;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::env;
use super::{Ltree, LtreeExtensions, subltree, subpath, nlevel, index, text2ltree, ltree2text};

table! {
    use super::Ltree;
    use diesel::types::*;

    my_tree (id) {
        id -> Int4,
        path -> Ltree,
    }
}

#[derive(Queryable, Debug)]
struct MyTree {
    pub id: i32,
    pub path: String,
}

fn get_connection() -> PgConnection {
    dotenv::dotenv().ok();

    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect("Error connecting to TEST_DATABASE_URL")
}

#[test]
fn base_operations() {
    let connection = get_connection();

    let results = my_tree::table
        .select((my_tree::id, ltree2text(my_tree::path)))
        .filter(
            my_tree::path
                .contained_by(text2ltree("root.eukaryota.plantae"))
                .or(my_tree::path.contains(text2ltree("root.bacteria"))),
        )
        .order(my_tree::id)
        .load::<MyTree>(&connection)
        .unwrap()
        .into_iter()
        .map(|t| t.path)
        .collect::<Vec<_>>();

    assert_eq!(
        results,
        [
            "root",
            "root.bacteria",
            "root.eukaryota.plantae",
            "root.eukaryota.plantae.nematophyta",
            "root.eukaryota.plantae.chlorophyta",
        ]
    );
}


#[test]
fn functions() {
    let connection = get_connection();

    let result = select(ltree2text(subltree(text2ltree("Top.Child1.Child2"), 1, 2)))
        .get_result::<String>(&connection);

    assert_eq!(result, Ok("Child1".into()));

    let result = select(ltree2text(subpath(text2ltree("Top.Child1.Child2"), 0, 2)))
        .get_result::<String>(&connection);

    assert_eq!(result, Ok("Top.Child1".into()));

    let result = select(nlevel(text2ltree("Top.Child1.Child2"))).get_result::<i32>(&connection);

    assert_eq!(result, Ok(3));

    let result = select(index(
        text2ltree("0.1.2.3.5.4.5.6.8.5.6.8"),
        text2ltree("5.6"),
    )).get_result::<i32>(&connection);

    assert_eq!(result, Ok(6));
}
