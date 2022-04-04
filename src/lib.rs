extern crate byteorder;
#[macro_use]
extern crate diesel;

#[cfg(test)]
mod tests;

pub mod sql_types {
    #[derive(SqlType, QueryId)]
    #[postgres(type_name = "ltree")]
    pub struct Ltree;

    #[derive(SqlType, Clone, Copy, QueryId)]
    #[postgres(type_name = "lquery")]
    pub struct Lquery;

    #[derive(SqlType, Clone, Copy, QueryId)]
    #[postgres(type_name = "ltxtquery")]
    pub struct Ltxtquery;
}

pub mod values {
    use std::io::Read;

    use byteorder::ReadBytesExt;
    use diesel::deserialize;
    use diesel::pg::Pg;
    use diesel::sql_types::Text;

    #[derive(Debug, PartialEq, Clone, FromSqlRow, AsExpression)]
    #[sql_type = "crate::sql_types::Ltree"]
    pub struct Ltree(pub String);

    impl diesel::serialize::ToSql<crate::sql_types::Ltree, Pg> for Ltree {
        fn to_sql<W: std::io::Write>(
            &self,
            out: &mut diesel::serialize::Output<W, Pg>,
        ) -> diesel::serialize::Result {
            out.write_all(self.0.as_bytes())?;
            Ok(diesel::serialize::IsNull::No)
        }
    }

    impl diesel::deserialize::FromSql<crate::sql_types::Ltree, Pg> for Ltree {
        fn from_sql(raw: Option<&[u8]>) -> deserialize::Result<Self> {
            let mut raw = not_none!(raw);

            let version = raw.read_i8()?;
            debug_assert_eq!(version, 1, "Unknown ltree binary protocol version.");

            let mut buf = String::new();
            raw.read_to_string(&mut buf)?;
            Ok(Ltree(buf))
        }
    }

    impl<DB> diesel::serialize::ToSql<Text, DB> for Ltree
    where
        String: diesel::serialize::ToSql<Text, DB>,
        DB: diesel::backend::Backend,
        DB: diesel::sql_types::HasSqlType<crate::sql_types::Ltree>,
    {
        fn to_sql<W: std::io::Write>(
            &self,
            out: &mut diesel::serialize::Output<W, DB>,
        ) -> diesel::serialize::Result {
            self.0.to_sql(out)
        }
    }

    impl<DB> diesel::deserialize::FromSql<Text, DB> for Ltree
    where
        String: diesel::deserialize::FromSql<Text, DB>,
        DB: diesel::backend::Backend,
        DB: diesel::sql_types::HasSqlType<crate::sql_types::Ltree>,
    {
        fn from_sql(raw: Option<&DB::RawValue>) -> deserialize::Result<Self> {
            String::from_sql(raw).map(Ltree)
        }
    }
}

pub mod functions {
    use crate::sql_types::*;
    use diesel::sql_types::*;

    sql_function!(fn subltree(ltree: Ltree, start: Int4, end: Int4) -> Ltree);
    sql_function!(fn subpath(ltree: Ltree, offset: Int4, len: Int4) -> Ltree);
    // sql_function!(fn subpath(ltree: Ltree, offset: Int4) -> Ltree);
    sql_function!(fn nlevel(ltree: Ltree) -> Int4);
    //sql_function!(fn index(a: Ltree, b: Ltree) -> Int4);
    sql_function!(fn index(a: Ltree, b: Ltree, offset: Int4) -> Int4);
    sql_function!(fn text2ltree(text: Text) -> Ltree);
    sql_function!(fn ltree2text(ltree: Ltree) -> Text);
    sql_function!(fn lca(ltrees: Array<Ltree>) -> Ltree);

    sql_function!(fn lquery(x: Text) -> Lquery);
    sql_function!(fn ltxtquery(x: Text) -> Ltxtquery);
}

pub mod dsl {
    use crate::sql_types::*;
    use diesel::expression::{AsExpression, Expression};
    use diesel::sql_types::Array;

    mod predicates {
        use crate::sql_types::*;
        use diesel::pg::Pg;

        diesel_infix_operator!(Contains, " @> ", backend: Pg);
        diesel_infix_operator!(ContainedBy, " <@ ", backend: Pg);
        diesel_infix_operator!(Matches, " ~ ", backend: Pg);
        diesel_infix_operator!(MatchesAny, " ? ", backend: Pg);
        diesel_infix_operator!(TMatches, " @ ", backend: Pg);
        diesel_infix_operator!(Concat, " || ", Ltree, backend: Pg);
        diesel_infix_operator!(FirstContains, " ?@> ", Ltree, backend: Pg);
        diesel_infix_operator!(FirstContainedBy, " ?<@ ", Ltree, backend: Pg);
        diesel_infix_operator!(FirstMatches, " ?~ ", Ltree, backend: Pg);
        diesel_infix_operator!(FirstTMatches, " ?@ ", Ltree, backend: Pg);
    }

    use self::predicates::*;

    pub trait LtreeExtensions: Expression<SqlType = Ltree> + Sized {
        fn contains<T: AsExpression<Ltree>>(self, other: T) -> Contains<Self, T::Expression> {
            Contains::new(self, other.as_expression())
        }

        fn contains_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> Contains<Self, T::Expression> {
            Contains::new(self, other.as_expression())
        }

        fn contained_by<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> ContainedBy<Self, T::Expression> {
            ContainedBy::new(self, other.as_expression())
        }

        fn contained_by_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> ContainedBy<Self, T::Expression> {
            ContainedBy::new(self, other.as_expression())
        }

        fn matches<T: AsExpression<Lquery>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }

        fn matches_any<T: AsExpression<Array<Lquery>>>(
            self,
            other: T,
        ) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }

        fn tmatches<T: AsExpression<Ltxtquery>>(self, other: T) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }

        fn concat<T: AsExpression<Ltree>>(self, other: T) -> Concat<Self, T::Expression> {
            Concat::new(self, other.as_expression())
        }
    }

    pub trait LtreeArrayExtensions: Expression<SqlType = Array<Ltree>> + Sized {
        fn any_contains<T: AsExpression<Ltree>>(self, other: T) -> Contains<Self, T::Expression> {
            Contains::new(self, other.as_expression())
        }

        fn any_contained_by<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> ContainedBy<Self, T::Expression> {
            ContainedBy::new(self, other.as_expression())
        }

        fn any_matches<T: AsExpression<Lquery>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }

        fn any_matches_any<T: AsExpression<Array<Lquery>>>(
            self,
            other: T,
        ) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }

        fn any_tmatches<T: AsExpression<Ltxtquery>>(
            self,
            other: T,
        ) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }

        fn first_contains<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> FirstContains<Self, T::Expression> {
            FirstContains::new(self, other.as_expression())
        }

        fn first_contained_by<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> FirstContainedBy<Self, T::Expression> {
            FirstContainedBy::new(self, other.as_expression())
        }

        fn first_matches<T: AsExpression<Lquery>>(
            self,
            other: T,
        ) -> FirstMatches<Self, T::Expression> {
            FirstMatches::new(self, other.as_expression())
        }

        fn first_tmatches<T: AsExpression<Ltxtquery>>(
            self,
            other: T,
        ) -> FirstTMatches<Self, T::Expression> {
            FirstTMatches::new(self, other.as_expression())
        }
    }

    pub trait LqueryExtensions: Expression<SqlType = Lquery> + Sized {
        fn matches<T: AsExpression<Ltree>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }

        fn matches_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
    }

    pub trait LqueryArrayExtensions: Expression<SqlType = Array<Lquery>> + Sized {
        fn any_matches<T: AsExpression<Ltree>>(self, other: T) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }

        fn any_matches_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }
    }

    pub trait LtxtqueryExtensions: Expression<SqlType = Ltxtquery> + Sized {
        fn tmatches<T: AsExpression<Ltree>>(self, other: T) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }

        fn tmatches_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }
    }

    impl<T: Expression<SqlType = Ltree>> LtreeExtensions for T {}
    impl<T: Expression<SqlType = Array<Ltree>>> LtreeArrayExtensions for T {}
    impl<T: Expression<SqlType = Lquery>> LqueryExtensions for T {}
    impl<T: Expression<SqlType = Array<Lquery>>> LqueryArrayExtensions for T {}
    impl<T: Expression<SqlType = Ltxtquery>> LtxtqueryExtensions for T {}
}

pub use crate::dsl::*;
pub use crate::functions::*;
pub use crate::values::*;
