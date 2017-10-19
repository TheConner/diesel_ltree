#[macro_use]
extern crate diesel;
#[cfg(test)]
#[macro_use]
extern crate diesel_codegen;

mod types {
    use diesel::pg::{Pg, PgMetadataLookup, PgTypeMetadata};
    use diesel::types::HasSqlType;

    #[derive(Clone, Copy)]
    pub struct Ltree;

    impl HasSqlType<Ltree> for Pg {
        fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
            PgTypeMetadata {
                oid: 24754,
                array_oid: 24757,
            }
        }
    }
}

mod functions {
    use types::*;
    use diesel::types::*;

    sql_function!(ltree2text, ltree2text_t, (x: Ltree) -> Text);
    sql_function!(text2ltree, text2ltree_t, (x: Text) -> Ltree);
}

mod dsl {
    use types::*;
    use diesel::expression::{AsExpression, Expression};

    mod predicates {
        use diesel::pg::Pg;

        diesel_infix_operator!(Contains, " @> ", backend: Pg);
        diesel_infix_operator!(ContainedBy, " <@ ", backend: Pg);
    }

    use self::predicates::*;

    pub trait LtreeExtensions: Expression<SqlType = Ltree> + Sized {
        fn contains<T: AsExpression<Ltree>>(self, other: T) -> Contains<Self, T::Expression> {
            Contains::new(self, other.as_expression())
        }

        fn contained_by<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> ContainedBy<Self, T::Expression> {
            ContainedBy::new(self, other.as_expression())
        }
    }

    impl<T: Expression<SqlType = Ltree>> LtreeExtensions for T {}
}

#[cfg(test)]
mod tests {
    extern crate dotenv;

    use diesel::prelude::*;
    use diesel::pg::PgConnection;
    use std::env;
    use super::{Ltree, LtreeExtensions, ltree2text, text2ltree};

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

    #[test]
    fn base_test() {
        use self::my_tree;
        use self::my_tree::dsl::*;

        dotenv::dotenv().ok();

        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let connection =
            PgConnection::establish(&database_url).expect("Error connecting to TEST_DATABASE_URL");

        let results = my_tree
            .select((my_tree::id, ltree2text(my_tree::path)))
            .filter(my_tree::path.contained_by(text2ltree("root.eukaryota.plantae")))
            .order(my_tree::id)
            .load::<MyTree>(&connection)
            .unwrap()
            .into_iter()
            .map(|t| t.path)
            .collect::<Vec<_>>();

        assert_eq!(
            results,
            [
                "root.eukaryota.plantae",
                "root.eukaryota.plantae.nematophyta",
                "root.eukaryota.plantae.chlorophyta"
            ]
        );
    }
}

pub use self::types::*;
pub use self::functions::*;
pub use self::dsl::*;
