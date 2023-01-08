// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ltree"))]
    pub struct Ltree;
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    my_tree (id) {
        id -> Int4,
        path -> Ltree,
    }
}
