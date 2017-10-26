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
        fn metadata(lookup: &PgMetadataLookup) -> PgTypeMetadata {
            lookup.lookup_type("ltree")
        }
    }

    impl_query_id!(Ltree);

    #[derive(Clone, Copy)]
    pub struct Lquery;

    impl HasSqlType<Lquery> for Pg {
        fn metadata(lookup: &PgMetadataLookup) -> PgTypeMetadata {
            lookup.lookup_type("lquery")
        }
    }

    impl_query_id!(Lquery);


    #[derive(Clone, Copy)]
    pub struct Ltxtquery;

    impl HasSqlType<Ltxtquery> for Pg {
        fn metadata(lookup: &PgMetadataLookup) -> PgTypeMetadata {
            lookup.lookup_type("ltxtquery")
        }
    }

    impl_query_id!(Ltxtquery);
}

mod functions {
    use types::*;
    use diesel::{AppearsOnTable, QueryResult, SelectableExpression};
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

    macro_rules! coerce_from_type {
        ($fn_name:ident, $struct_name:ident, $type:ty, $result:ty) => {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy)]
            #[doc(hidden)]
            pub struct $struct_name<T>(T);

            #[allow(non_camel_case_types)]
            pub type $fn_name<T> = $struct_name<<T as AsExpression<$type>>::Expression>;

            pub fn $fn_name<T>(expr: T) -> $fn_name<T> where T: AsExpression<$type> {
                $struct_name(expr.as_expression())
            }

            impl<T> Expression for $struct_name<T> where T: AsExpression<$type> {
                type SqlType = $result;
            }

            impl<T, QS> SelectableExpression<QS> for $struct_name<T>
                where
                T: AsExpression<$type>,
            {
            }

            impl<T, QS> AppearsOnTable<QS> for $struct_name<T>
                where
                T: AsExpression<$type>,
            {
            }

            impl_query_id!($struct_name<T>);

            impl<T, DB> QueryFragment<DB> for $struct_name<T>
                where
                DB: Backend,
            for<'a> (&'a T): QueryFragment<DB>,
            {
                fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
                    out.push_sql("(");
                    QueryFragment::walk_ast(&&self.0, out.reborrow())?;
                    out.push_sql(concat!(")::", stringify!($fn_name)));
                    Ok(())

                }
            }

            impl<T> NonAggregate for $struct_name<T>
                where
                T: NonAggregate,
            $struct_name<T>: Expression,
            {
            }
        }
    }

    coerce_from_type!(lquery, lquery_t, Text, Lquery);
    coerce_from_type!(ltxtquery, ltxtquery_t, Text, Ltxtquery);
}

mod dsl {
    use types::*;
    use diesel::expression::{AsExpression, Expression};

    mod predicates {
        use types::*;
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
        diesel_infix_operator!(TMatches, " @ ", backend: Pg);
        diesel_infix_operator!(Concat, " || ", Ltree, backend: Pg);
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

        fn tmatches<T: AsExpression<Ltxtquery>>(self, other: T) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }

        fn concat<T: AsExpression<Ltree>>(self, other: T) -> Concat<Self, T::Expression> {
            Concat::new(self, other.as_expression())
        }
    }

    pub trait LqueryExtensions: Expression<SqlType = Lquery> + Sized {
        fn matches<T: AsExpression<Ltree>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
    }

    pub trait LtxtqueryExtensions: Expression<SqlType = Ltxtquery> + Sized {
        fn tmatches<T: AsExpression<Ltree>>(self, other: T) -> TMatches<Self, T::Expression> {
            TMatches::new(self, other.as_expression())
        }
    }

    impl<T: Expression<SqlType = Ltree>> LtreeExtensions for T {}
    impl<T: Expression<SqlType = Lquery>> LqueryExtensions for T {}
    impl<T: Expression<SqlType = Ltxtquery>> LtxtqueryExtensions for T {}
}

pub use self::types::*;
pub use self::functions::*;
pub use self::dsl::*;
