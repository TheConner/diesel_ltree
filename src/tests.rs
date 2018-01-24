extern crate dotenv;

use diesel::select;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::env;
use super::{index, lquery, ltxtquery, nlevel, subltree, subpath, LqueryArrayExtensions,
            LqueryExtensions, Ltree, LtreeExtensions, LtxtqueryExtensions, ltree2text, text2ltree};

table! {
    use super::Ltree;
    use diesel::sql_types::*;

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
            "root.eukaryota.plantae.chlorophyta"
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

#[test]
fn operators() {
    use diesel::dsl::array;

    let connection = get_connection();

    let result = select((
        text2ltree("1.1").eq(text2ltree("1.2")),
        text2ltree("1.1").eq(text2ltree("1.1")),
        text2ltree("1.1").ne(text2ltree("1.2")),
        text2ltree("1.1").ne(text2ltree("1.1")),
    )).get_result::<(bool, bool, bool, bool)>(&connection);
    assert_eq!(result, Ok((false, true, true, false)));

    let result = select((
        text2ltree("1").lt(text2ltree("1.1")),
        text2ltree("1.2").gt(text2ltree("1.1")),
        text2ltree("1.2").le(text2ltree("1.1")),
        text2ltree("1.2.1").ge(text2ltree("1.2")),
    )).get_result::<(bool, bool, bool, bool)>(&connection);
    assert_eq!(result, Ok((true, true, false, true)));

    let result = select((
        text2ltree("foo_bar_baz").matches(lquery("foo_bar%")),
        text2ltree("foo_barbaz").matches(lquery("foo_bar%")),
        lquery("foo_bar%*").matches(text2ltree("foo1_bar2_baz")),
        lquery("foo_bar%*").matches(text2ltree("foo1_br2_baz")),
    )).get_result::<(bool, bool, bool, bool)>(&connection);
    assert_eq!(result, Ok((true, false, true, false)));

    let result = select((
        text2ltree("foo_bar_baz").matches_any(array((lquery("foo_bar%"), lquery("foo_bat%")))),
        text2ltree("foo_bar_baz").matches_any(array((lquery("foo_bat%"),))),
    )).get_result::<(bool, bool)>(&connection);
    assert_eq!(result, Ok((true, false)));

    let result = select((
        array((lquery("foo_bar%"), lquery("foo_bat%"))).matches_any(text2ltree("foo_bar_baz")),
        array((lquery("foo_bat%"),)).matches_any(text2ltree("foo_bar_baz")),
    )).get_result::<(bool, bool)>(&connection);
    assert_eq!(result, Ok((true, false)));

    let q = ltxtquery("Europe & Russia*@ & !Transportation");
    let result = select((
        text2ltree("Russian.Hello.Europe").tmatches(q),
        q.tmatches(text2ltree("Europe.russia.Transportation")),
        q.tmatches(text2ltree("russians.today.Europe")),
    )).get_result::<(bool, bool, bool)>(&connection);
    assert_eq!(result, Ok((true, false, true)));

    let result = select(ltree2text(text2ltree("a.b").concat(text2ltree("c.d"))))
        .get_result::<String>(&connection);
    assert_eq!(result, Ok("a.b.c.d".into()));
}
