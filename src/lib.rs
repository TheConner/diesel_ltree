//! diesel_ltree provides support for Postgres's 
//! [ltree](https://www.postgresql.org/docs/current/ltree.html) extension, 
//! including all of the operations and functions for working with hierarchial 
//! data in Postgres.
extern crate byteorder;
#[macro_use]
extern crate diesel;

#[cfg(test)]
mod tests;

pub mod sql_types {
    use diesel::query_builder::QueryId;
    use diesel::sql_types::SqlType;

    #[derive(SqlType, Clone, Copy, QueryId)]
    #[diesel(postgres_type(name = "ltree"))]
    pub struct Ltree;

    #[derive(SqlType, Clone, Copy, QueryId)]
    #[diesel(postgres_type(name = "lquery"))]
    pub struct Lquery;

    #[derive(SqlType, Clone, Copy, QueryId)]
    #[diesel(postgres_type(name = "ltxtquery"))]
    pub struct Ltxtquery;
}

pub mod values {
    use std::io::{Read, Write};

    use byteorder::{ReadBytesExt, WriteBytesExt};
    use diesel::deserialize::{self, FromSqlRow};
    use diesel::expression::AsExpression;
    use diesel::pg::{Pg, PgValue};
    use diesel::sql_types::Text;

    /// A ltree [label path](https://www.postgresql.org/docs/current/ltree.html#LTREE-DEFINITIONS).
    #[derive(Debug, PartialEq, Eq, Clone, FromSqlRow, AsExpression)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    #[diesel(sql_type = crate::sql_types::Ltree)]
    pub struct Ltree(pub String);

    impl diesel::serialize::ToSql<crate::sql_types::Ltree, Pg> for Ltree {
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, Pg>,
        ) -> diesel::serialize::Result {
            out.write_i8(1)?;
            out.write_all(self.0.as_bytes())?;
            Ok(diesel::serialize::IsNull::No)
        }
    }

    impl diesel::deserialize::FromSql<crate::sql_types::Ltree, Pg> for Ltree {
        fn from_sql(value: PgValue) -> deserialize::Result<Self> {
            let mut raw = value.as_bytes();

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
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, DB>,
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
        fn from_sql(
            bytes: <DB as diesel::backend::Backend>::RawValue<'_>,
        ) -> deserialize::Result<Self> {
            String::from_sql(bytes).map(Ltree)
        }
    }
}

pub mod functions {
    use crate::sql_types::*;
    use diesel::sql_types::*;

    define_sql_function!(fn subltree(ltree: Ltree, start: Int4, end: Int4) -> Ltree);
    define_sql_function!(fn subpath(ltree: Ltree, offset: Int4, len: Int4) -> Ltree);
    // define_sql_function!(fn subpath(ltree: Ltree, offset: Int4) -> Ltree);
    define_sql_function!(fn nlevel(ltree: Ltree) -> Int4);
    //define_sql_function!(fn index(a: Ltree, b: Ltree) -> Int4);
    define_sql_function!(fn index(a: Ltree, b: Ltree, offset: Int4) -> Int4);
    define_sql_function!(fn text2ltree(text: Text) -> Ltree);
    define_sql_function!(fn ltree2text(ltree: Ltree) -> Text);
    define_sql_function!(fn lca(ltrees: Array<Ltree>) -> Ltree);

    define_sql_function!(fn lquery(x: Text) -> Lquery);
    define_sql_function!(fn ltxtquery(x: Text) -> Ltxtquery);
}

pub mod dsl {
    use crate::sql_types::*;
    use diesel::expression::{AsExpression, Expression};
    use diesel::sql_types::Array;

    mod predicates {
        use crate::sql_types::*;
        use diesel::pg::Pg;

        diesel::infix_operator!(Contains, " @> ", backend: Pg);
        diesel::infix_operator!(ContainedBy, " <@ ", backend: Pg);
        diesel::infix_operator!(Matches, " ~ ", backend: Pg);
        diesel::infix_operator!(MatchesAny, " ? ", backend: Pg);
        diesel::infix_operator!(TMatches, " @ ", backend: Pg);
        diesel::infix_operator!(Concat, " || ", Ltree, backend: Pg);
        diesel::infix_operator!(FirstContains, " ?@> ", Ltree, backend: Pg);
        diesel::infix_operator!(FirstContainedBy, " ?<@ ", Ltree, backend: Pg);
        diesel::infix_operator!(FirstMatches, " ?~ ", Ltree, backend: Pg);
        diesel::infix_operator!(FirstTMatches, " ?@ ", Ltree, backend: Pg);
    }

    use self::predicates::*;

    /// Adds Ltree-specific extensions to queries.
    pub trait LtreeExtensions: Expression<SqlType = Ltree> + Sized {
        /// Checks if the current expression contains another Ltree expression.
        fn contains<T: AsExpression<Ltree>>(self, other: T) -> Contains<Self, T::Expression> {
            Contains::new(self, other.as_expression())
        }

        /// Checks if the current expression contains any Ltree expression in the given array.
        fn contains_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> Contains<Self, T::Expression> {
            Contains::new(self, other.as_expression())
        }

        /// Checks if the current expression is contained by another Ltree expression.
        fn contained_by<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> ContainedBy<Self, T::Expression> {
            ContainedBy::new(self, other.as_expression())
        }

        /// Checks if the current expression is contained by any Ltree expression in the given array.
        fn contained_by_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> ContainedBy<Self, T::Expression> {
            ContainedBy::new(self, other.as_expression())
        }

        /// Checks if the current expression matches another Lquery expression.
        fn matches<T: AsExpression<Lquery>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }

        /// Checks if the current expression matches any Lquery expression in the given array.
        fn matches_any<T: AsExpression<Array<Lquery>>>(
            self,
            other: T,
        ) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }

        /// Checks if the current expression matches another Ltxtquery expression.-
        fn tmatches<T: AsExpression<Ltxtquery>>(self, other: T) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }

        /// Concatenates the current expression with another Ltree expression.
        fn concat<T: AsExpression<Ltree>>(self, other: T) -> Concat<Self, T::Expression> {
            Concat::new(self, other.as_expression())
        }
    }

    /// Adds Ltree-specific extensions to arrays of Ltree expressions.
    pub trait LtreeArrayExtensions: Expression<SqlType = Array<Ltree>> + Sized {
        /// Checks if any Ltree expression in the array contains the specified Ltree expression.
        fn any_contains<T: AsExpression<Ltree>>(self, other: T) -> Contains<Self, T::Expression> {
            Contains::new(self, other.as_expression())
        }
        
        /// Checks if any Ltree expression in the array is contained by the specified Ltree expression.
        fn any_contained_by<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> ContainedBy<Self, T::Expression> {
            ContainedBy::new(self, other.as_expression())
        }

        /// Checks if any Ltree expression in the array matches the specified Lquery expression.
        fn any_matches<T: AsExpression<Lquery>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }

        /// Checks if any Ltree expression in the array matches any Lquery expression in the given array.
        fn any_matches_any<T: AsExpression<Array<Lquery>>>(
            self,
            other: T,
        ) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }
        
        /// Checks if any Ltree expression in the array matches the specified Ltxtquery expression.
        fn any_tmatches<T: AsExpression<Ltxtquery>>(
            self,
            other: T,
        ) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }

        /// Checks if the first Ltree expression in the array contains the specified Ltree expression.
        fn first_contains<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> FirstContains<Self, T::Expression> {
            FirstContains::new(self, other.as_expression())
        }
        
        /// Checks if the first Ltree expression in the array is contained by the specified Ltree expression.
        fn first_contained_by<T: AsExpression<Ltree>>(
            self,
            other: T,
        ) -> FirstContainedBy<Self, T::Expression> {
            FirstContainedBy::new(self, other.as_expression())
        }

        /// Checks if the first Ltree expression in the array matches the specified Lquery expression.
        fn first_matches<T: AsExpression<Lquery>>(
            self,
            other: T,
        ) -> FirstMatches<Self, T::Expression> {
            FirstMatches::new(self, other.as_expression())
        }

        /// Checks if the first Ltree expression in the array matches the specified Ltxtquery expression.
        fn first_tmatches<T: AsExpression<Ltxtquery>>(
            self,
            other: T,
        ) -> FirstTMatches<Self, T::Expression> {
            FirstTMatches::new(self, other.as_expression())
        }
    }

    /// Implements lquery extensions for diesel queries
    pub trait LqueryExtensions: Expression<SqlType = Lquery> + Sized {
        /// Checks if the current Lquery expression matches the specified Ltree expression.
        fn matches<T: AsExpression<Ltree>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
        
        /// Checks if the current Lquery expression matches any Ltree expression in the given array.
        fn matches_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
    }
    
    /// Adds Lquery-specific extensions to arrays of Lquery expressions.
    pub trait LqueryArrayExtensions: Expression<SqlType = Array<Lquery>> + Sized {
        /// Checks if any Lquery expression in the array matches the specified Ltree expression.
        fn any_matches<T: AsExpression<Ltree>>(self, other: T) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }

        /// Checks if any Lquery expression in the array matches any Ltree expression in the given array.
        fn any_matches_any<T: AsExpression<Array<Ltree>>>(
            self,
            other: T,
        ) -> MatchesAny<Self, T::Expression> {
            MatchesAny::new(self, other.as_expression())
        }
    }

    /// A trait for adding Ltxtquery-specific extensions to queries.
    pub trait LtxtqueryExtensions: Expression<SqlType = Ltxtquery> + Sized {
        /// Checks if the current Ltxtquery expression matches the specified Ltree expression.
        fn tmatches<T: AsExpression<Ltree>>(self, other: T) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }
        
        /// Checks if the current Ltxtquery expression matches any Ltree expression in the given array.
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
