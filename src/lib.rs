// #[macro_use]
extern crate diesel;

mod types {
    use diesel::types::{HasSqlType};
    use diesel::pg::{Pg, PgTypeMetadata, PgMetadataLookup};

    #[derive(Clone, Copy)] pub struct Ltree;

    impl HasSqlType<Ltree> for Pg {
        fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
            PgTypeMetadata {
                oid: 24754,
                array_oid: 24757,
            }
        }
    }
}

pub use self::types::*;
