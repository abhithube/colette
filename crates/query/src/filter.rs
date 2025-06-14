use colette_core::filter::{BooleanOp, DateOp, NumberOp, TextOp};
use sea_query::{Expr, ExprTrait, SimpleExpr};

pub(crate) trait ToColumn {
    fn to_column(self) -> Expr;
}

pub(crate) trait ToSql {
    fn to_sql(self) -> SimpleExpr;
}

impl ToSql for (Expr, TextOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            TextOp::Equals(value) => column.eq(value),
            TextOp::Contains(value) => column.like(format!("%{value}%")),
            TextOp::StartsWith(value) => column.like(format!("{value}%")),
            TextOp::EndsWith(value) => column.like(format!("%{value}")),
        }
    }
}

impl ToSql for (Expr, NumberOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            NumberOp::Equals(value) => column.eq(value),
            NumberOp::LessThan(value) => column.lt(value),
            NumberOp::GreaterThan(value) => column.gt(value),
            NumberOp::Between(value) => column.between(value.start, value.end),
        }
    }
}

impl ToSql for (Expr, BooleanOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            BooleanOp::Equals(value) => column.eq(value),
        }
    }
}

impl ToSql for (Expr, DateOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            DateOp::Before(value) => column.lt(value),
            DateOp::After(value) => column.gt(value),
            DateOp::Between(value) => column.between(value.start, value.end),
            DateOp::InLast(value) => Expr::cust("extract(epoch from now())")
                .sub(column)
                .lt(value),
        }
    }
}
