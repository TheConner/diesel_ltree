extern crate dotenv;

use diesel::debug_query;
use diesel::dsl::array;
use diesel::pg::Pg;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::select;
use std::env;

use crate::{
    index, lca, lquery, ltree2text, ltxtquery, nlevel, subltree, subpath, text2ltree,
    LqueryArrayExtensions, LqueryExtensions, Ltree, LtreeArrayExtensions, LtreeExtensions,
    LtxtqueryExtensions,
};

table! {
    use crate::sql_types::Ltree;
    use diesel::sql_types::*;

    my_tree (id) {
        id -> Int4,
        path -> Ltree,
    }
}

#[derive(Queryable, Debug)]
struct MyTree {
    #[allow(dead_code)]
    pub id: i32,
    pub path: Ltree,
}

fn get_connection() -> PgConnection {
    dotenv::dotenv().ok();

    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let mut connection = PgConnection::establish(&database_url).expect("Error connecting to TEST_DATABASE_URL");
    connection.begin_test_transaction().expect("couldn't begin test transaction");
    connection
}

#[test]
fn base_operations_without_converting_to_text_first() {
    let mut connection = get_connection();

    let results = my_tree::table
        .select((my_tree::id, my_tree::path))
        .filter(
            my_tree::path
                .contained_by(text2ltree("root.eukaryota.plantae"))
                .or(my_tree::path.contains(text2ltree("root.bacteria"))),
        )
        .order(my_tree::id)
        .load::<MyTree>(&mut connection)
        .unwrap()
        .into_iter()
        .map(|t| t.path)
        .collect::<Vec<_>>();

    assert_eq!(
        results,
        [
            Ltree("root".to_string()),
            Ltree("root.bacteria".to_string()),
            Ltree("root.eukaryota.plantae".to_string()),
            Ltree("root.eukaryota.plantae.nematophyta".to_string()),
            Ltree("root.eukaryota.plantae.chlorophyta".to_string())
        ]
    );
}

#[test]
fn base_operations() {
    let mut connection = get_connection();

    let results = my_tree::table
        .select((my_tree::id, ltree2text(my_tree::path)))
        .filter(
            my_tree::path
                .contained_by(text2ltree("root.eukaryota.plantae"))
                .or(my_tree::path.contains(text2ltree("root.bacteria"))),
        )
        .order(my_tree::id)
        .load::<MyTree>(&mut connection)
        .unwrap()
        .into_iter()
        .map(|t| t.path)
        .collect::<Vec<_>>();

    assert_eq!(
        results,
        [
            Ltree("root".to_string()),
            Ltree("root.bacteria".to_string()),
            Ltree("root.eukaryota.plantae".to_string()),
            Ltree("root.eukaryota.plantae.nematophyta".to_string()),
            Ltree("root.eukaryota.plantae.chlorophyta".to_string())
        ]
    );
}

#[test]
fn insert_query_generation() {
    let query = debug_query::<Pg, _>(&diesel::insert_into(my_tree::table).values(&vec![
        my_tree::path.eq(Ltree("root".to_string())),
        my_tree::path.eq(Ltree("root.bacteria".to_string())),
        my_tree::path.eq(Ltree("root.eukaryota.plantae".to_string())),
        my_tree::path.eq(Ltree("root.eukaryota.plantae.nematophyta".to_string())),
        my_tree::path.eq(Ltree("root.eukaryota.plantae.chlorophyta".to_string())),
    ]))
    .to_string();

    assert_eq!(
        query,
        "INSERT INTO \"my_tree\" (\"path\") VALUES ($1), ($2), ($3), ($4), ($5) -- binds: [Ltree(\"root\"), Ltree(\"root.bacteria\"), Ltree(\"root.eukaryota.plantae\"), Ltree(\"root.eukaryota.plantae.nematophyta\"), Ltree(\"root.eukaryota.plantae.chlorophyta\")]"
    );
}

#[test]
fn insert_works() {
    let mut connection = get_connection();

    let result = diesel::insert_into(my_tree::table)
        .values(&vec![
            my_tree::path.eq(Ltree("root".to_string())),
            my_tree::path.eq(Ltree("root.bacteria".to_string())),
            my_tree::path.eq(Ltree("root.eukaryota.plantae".to_string())),
            my_tree::path.eq(Ltree("root.eukaryota.plantae.nematophyta".to_string())),
            my_tree::path.eq(Ltree("root.eukaryota.plantae.chlorophyta".to_string())),
        ])
        .execute(&mut connection);

    assert_eq!(result, Ok(5));
}

#[test]
fn functions() {
    let mut connection = get_connection();

    let result = select(ltree2text(subltree(text2ltree("Top.Child1.Child2"), 1, 2)))
        .get_result::<String>(&mut connection);
    assert_eq!(result, Ok("Child1".into()));

    let result = select(ltree2text(subpath(text2ltree("Top.Child1.Child2"), 0, 2)))
        .get_result::<String>(&mut connection);
    assert_eq!(result, Ok("Top.Child1".into()));

    let result = select(nlevel(text2ltree("Top.Child1.Child2"))).get_result::<i32>(&mut connection);
    assert_eq!(result, Ok(3));

    let result = select(index(
        text2ltree("0.1.2.3.5.4.5.6.8.5.6.8"),
        text2ltree("5.6"),
        0,
    ))
    .get_result::<i32>(&mut connection);
    assert_eq!(result, Ok(6));

    let result = select(ltree2text(lca(array((
        text2ltree("1.2.2.3"),
        text2ltree("1.2.3"),
    )))))
    .get_result::<String>(&mut connection);
    assert_eq!(result, Ok("1.2".into()));
}

#[test]
fn operators() {
    let mut connection = get_connection();

    let result = select((
        text2ltree("1.1").eq(text2ltree("1.2")),
        text2ltree("1.1").eq(text2ltree("1.1")),
        text2ltree("1.1").ne(text2ltree("1.2")),
        text2ltree("1.1").ne(text2ltree("1.1")),
    ))
    .get_result::<(bool, bool, bool, bool)>(&mut connection);
    assert_eq!(result, Ok((false, true, true, false)));

    let result = select((
        text2ltree("1").lt(text2ltree("1.1")),
        text2ltree("1.2").gt(text2ltree("1.1")),
        text2ltree("1.2").le(text2ltree("1.1")),
        text2ltree("1.2.1").ge(text2ltree("1.2")),
    ))
    .get_result::<(bool, bool, bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, true, false, true)));

    let result = select((
        text2ltree("foo_bar_baz").matches(lquery("foo_bar%")),
        text2ltree("foo_barbaz").matches(lquery("foo_bar%")),
        lquery("foo_bar%*").matches(text2ltree("foo1_bar2_baz")),
        lquery("foo_bar%*").matches(text2ltree("foo1_br2_baz")),
    ))
    .get_result::<(bool, bool, bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, false, true, false)));

    let result = select((
        text2ltree("foo_bar_baz").matches_any(array((lquery("foo_bar%"), lquery("foo_bat%")))),
        text2ltree("foo_bar_baz").matches_any(array((lquery("foo_bat%"),))),
    ))
    .get_result::<(bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, false)));

    let result = select((
        array((lquery("foo_bar%"), lquery("foo_bat%"))).any_matches(text2ltree("foo_bar_baz")),
        array((lquery("foo_bat%"),)).any_matches(text2ltree("foo_bar_baz")),
    ))
    .get_result::<(bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, false)));

    let q = ltxtquery("Europe & Russia*@ & !Transportation");
    let result = select((
        text2ltree("Russian.Hello.Europe").tmatches(q),
        q.tmatches(text2ltree("Europe.russia.Transportation")),
        q.tmatches(text2ltree("russians.today.Europe")),
    ))
    .get_result::<(bool, bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, false, true)));

    let result = select(ltree2text(text2ltree("a.b").concat(text2ltree("c.d"))))
        .get_result::<String>(&mut connection);
    assert_eq!(result, Ok("a.b.c.d".into()));

    let result = select((
        text2ltree("a.b").contained_by_any(array((text2ltree("a"), text2ltree("a.b.c")))),
        array((text2ltree("a"), text2ltree("a.b.c"))).any_contains(text2ltree("a.b")),
        text2ltree("a.b").contains_any(array((text2ltree("a"), text2ltree("a.b.c")))),
        array((text2ltree("a"), text2ltree("a.b.c"))).any_contained_by(text2ltree("a.b")),
    ))
    .get_result::<(bool, bool, bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, true, true, true)));

    let result = select((
        array((text2ltree("a"), text2ltree("a.b"))).any_matches(lquery("a%")),
        lquery("a%").matches_any(array((text2ltree("a"), text2ltree("a.b")))),
    ))
    .get_result::<(bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, true)));

    let result = select((
        array((text2ltree("a"), text2ltree("a.b")))
            .any_matches_any(array((lquery("a%"), lquery("b%")))),
        array((lquery("a%"), lquery("b%")))
            .any_matches_any(array((text2ltree("a"), text2ltree("a.b")))),
    ))
    .get_result::<(bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, true)));

    let result = select((
        array((text2ltree("a"), text2ltree("a.b"))).any_tmatches(ltxtquery("a")),
        ltxtquery("a").tmatches_any(array((text2ltree("a"), text2ltree("a.b")))),
    ))
    .get_result::<(bool, bool)>(&mut connection);
    assert_eq!(result, Ok((true, true)));

    let result = select((
        ltree2text(array((text2ltree("a.b.c"), text2ltree("a"))).first_contains(text2ltree("a.b"))),
        ltree2text(
            array((text2ltree("a"), text2ltree("a.b.c"))).first_contained_by(text2ltree("a.b")),
        ),
    ))
    .get_result::<(String, String)>(&mut connection);
    assert_eq!(result, Ok(("a".into(), "a.b.c".into())));

    let result = select((
        ltree2text(array((text2ltree("a.b.c"), text2ltree("a"))).first_matches(lquery("a%"))),
        ltree2text(
            array((text2ltree("a"), text2ltree("a.b.c"))).first_tmatches(ltxtquery("a & b")),
        ),
    ))
    .get_result::<(String, String)>(&mut connection);
    assert_eq!(result, Ok(("a".into(), "a.b.c".into())));
}
