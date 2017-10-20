#[macro_use]
extern crate diesel;
#[cfg(test)]
#[macro_use]
extern crate diesel_codegen;

#[cfg(test)]
mod tests;

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

    impl_query_id!(Ltree);

    #[derive(Clone, Copy)]
    pub struct Lquery;

    impl HasSqlType<Lquery> for Pg {
        fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
            PgTypeMetadata {
                oid: 24808,
                array_oid: 24811,
            }
        }
    }

    impl_query_id!(Lquery);


    #[derive(Clone, Copy)]
    pub struct Ltxtquery;

    impl HasSqlType<Ltxtquery> for Pg {
        fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
            PgTypeMetadata {
                oid: 24824,
                array_oid: 24827,
            }
        }
    }

    impl_query_id!(Ltxtquery);
}

mod functions {
    use types::*;
    use diesel::{AppearsOnTable, SelectableExpression, QueryResult};
    use diesel::backend::Backend;
    use diesel::types::*;
    use diesel::expression::{AsExpression, Expression, NonAggregate};
    use diesel::query_builder::{AstPass, QueryFragment};

    sql_function!(subltree, subltree_t, (ltree: Ltree, start: Int4, end: Int4) -> Ltree);
    sql_function!(subpath, subpath_t, (ltree: Ltree, offset: Int4, len: Int4) -> Ltree);
    // sql_function!(subpath, subpath_t, (ltree: Ltree, offset: Int4) -> Ltree);
    sql_function!(nlevel, nlevel_t, (ltree: Ltree) -> Int4);
    sql_function!(index, index_t, (a: Ltree, b: Ltree) -> Int4);
    // sql_function!(index, index_t, (a: Ltree, b: Ltree, offset: Int4) -> Int4);
    sql_function!(text2ltree, text2ltree_t, (text: Text) -> Ltree);
    sql_function!(ltree2text, ltree2text_t, (ltree: Ltree) -> Text);

    pub struct LqueryFromTextS<T>(T);
    pub type LqueryFromText<T> = LqueryFromTextS<<T as AsExpression<Text>>::Expression>;

    pub fn lquery_from_text<T>(expr: T) -> LqueryFromText<T>
    where
        T: AsExpression<Text>,
    {
        LqueryFromTextS(expr.as_expression())
    }

    impl<T> Expression for LqueryFromTextS<T>
    where
        T: AsExpression<Text>,
    {
        type SqlType = Lquery;
    }

    impl<T, QS> SelectableExpression<QS> for LqueryFromTextS<T>
    where
        T: AsExpression<Text>,
    {
    }

    impl<T, QS> AppearsOnTable<QS> for LqueryFromTextS<T>
    where
        T: AsExpression<Text>,
    {
    }

    impl_query_id!(LqueryFromTextS<T>);

    impl<T, DB> QueryFragment<DB> for LqueryFromTextS<T>
    where
        DB: Backend,
        for<'a> (&'a T): QueryFragment<DB>,
    {
        fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
            out.push_sql("(");
            QueryFragment::walk_ast(&(&self.0), out.reborrow())?;
            out.push_sql(")::lquery");
            Ok(())

        }
    }

    impl<T> NonAggregate for LqueryFromTextS<T>
    where
        T: NonAggregate,
        LqueryFromTextS<T>: Expression,
    {
    }

}

mod dsl {
    use types::*;
    use diesel::expression::{AsExpression, Expression};

    mod predicates {
        use diesel::pg::Pg;

        diesel_infix_operator!(Contains, " @> ", backend: Pg);
        diesel_infix_operator!(ContainedBy, " <@ ", backend: Pg);
        diesel_infix_operator!(Eq, " = ", backend: Pg);
        diesel_infix_operator!(NotEq, " <> ", backend: Pg);
        diesel_infix_operator!(Gt, " > ", backend: Pg);
        diesel_infix_operator!(GtEq, " >= ", backend: Pg);
        diesel_infix_operator!(Lt, " < ", backend: Pg);
        diesel_infix_operator!(LtEq, " <= ", backend: Pg);
        diesel_infix_operator!(Matches, " ~ ", backend: Pg);
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

        fn eq<T: AsExpression<Ltree>>(self, other: T) -> Eq<Self, T::Expression> {
            Eq::new(self, other.as_expression())
        }

        fn ne<T: AsExpression<Ltree>>(self, other: T) -> NotEq<Self, T::Expression> {
            NotEq::new(self, other.as_expression())
        }

        fn gt<T: AsExpression<Ltree>>(self, other: T) -> Gt<Self, T::Expression> {
            Gt::new(self, other.as_expression())
        }

        fn ge<T: AsExpression<Ltree>>(self, other: T) -> GtEq<Self, T::Expression> {
            GtEq::new(self, other.as_expression())
        }

        fn lt<T: AsExpression<Ltree>>(self, other: T) -> Lt<Self, T::Expression> {
            Lt::new(self, other.as_expression())
        }

        fn le<T: AsExpression<Ltree>>(self, other: T) -> LtEq<Self, T::Expression> {
            LtEq::new(self, other.as_expression())
        }

        fn matches<T: AsExpression<Lquery>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
    }

    pub trait LqueryExtensions: Expression<SqlType = Lquery> + Sized {
        fn matches<T: AsExpression<Ltree>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
    }

    impl<T: Expression<SqlType = Ltree>> LtreeExtensions for T {}

    impl<T: Expression<SqlType = Lquery>> LqueryExtensions for T {}
}

pub use self::types::*;
pub use self::functions::*;
pub use self::dsl::*;
