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
            TextOp::Equals(value) => column.to_owned().eq(value),
            TextOp::Contains(value) => column.to_owned().like(format!("%{}%", value)),
            TextOp::StartsWith(value) => column.to_owned().like(format!("{}%", value)),
            TextOp::EndsWith(value) => column.to_owned().like(format!("%{}", value)),
        }
    }
}

impl ToSql for (Expr, NumberOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            NumberOp::Equals(value) => column.to_owned().eq(value),
            NumberOp::LessThan(value) => column.to_owned().lt(value),
            NumberOp::GreaterThan(value) => column.to_owned().gt(value),
            NumberOp::Between(value) => column.to_owned().between(value.start, value.end),
        }
    }
}

impl ToSql for (Expr, BooleanOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            BooleanOp::Equals(value) => column.to_owned().eq(value),
        }
    }
}

impl ToSql for (Expr, DateOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            DateOp::Before(value) => column.to_owned().lt(value.timestamp()),
            DateOp::After(value) => column.to_owned().gt(value.timestamp()),
            DateOp::Between(value) => column
                .to_owned()
                .between(value.start.timestamp(), value.end.timestamp()),
            DateOp::InLast(value) => Expr::cust("strftime('%s', 'now')")
                .sub(column.to_owned())
                .lt(value),
        }
    }
}
