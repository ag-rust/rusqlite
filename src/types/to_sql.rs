use super::{Null, Value, ValueRef};
use ::Result;

pub enum ToSqlOutput<'a> {
    Borrowed(ValueRef<'a>),
    Owned(Value),

    #[cfg(feature = "blob")]
    ZeroBlob(i32),
}

impl<'a, T: ?Sized> From<&'a T> for ToSqlOutput<'a> where &'a T: Into<ValueRef<'a>> {
    fn from(t: &'a T) -> Self {
        ToSqlOutput::Borrowed(t.into())
    }
}

impl<'a, T: Into<Value>> From<T> for ToSqlOutput<'a> {
    fn from(t: T) -> Self {
        ToSqlOutput::Owned(t.into())
    }
}

/// A trait for types that can be converted into SQLite values.
pub trait ToSql {
    fn to_sql(&self) -> Result<ToSqlOutput>;
}

// We should be able to use a generic impl like this:
//
// impl<T: Copy> ToSql for T where T: Into<Value> {
//     fn to_sql(&self) -> Result<ToSqlOutput> {
//         Ok(ToSqlOutput::from((*self).into()))
//     }
// }
//
// instead of the following macro, but this runs afoul of
// https://github.com/rust-lang/rust/issues/30191 and reports conflicting
// implementations even when there aren't any.

macro_rules! to_sql_self(
    ($t:ty) => (
        impl ToSql for $t {
            fn to_sql(&self) -> Result<ToSqlOutput> {
                Ok(ToSqlOutput::from(*self))
            }
        }
    )
);

to_sql_self!(Null);
to_sql_self!(bool);
to_sql_self!(i32);
to_sql_self!(i64);
to_sql_self!(f64);

impl<'a, T: ?Sized> ToSql for &'a T where &'a T: Into<ToSqlOutput<'a>> {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        Ok(ToSqlOutput::from((*self).into()))
    }
}

impl ToSql for String {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl ToSql for Vec<u8> {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        Ok(ToSqlOutput::from(self.as_slice()))
    }
}

impl ToSql for Value {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        Ok(ToSqlOutput::from(self))
    }
}

impl<T: ToSql> ToSql for Option<T> {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        match *self {
            None => Ok(ToSqlOutput::from(Null)),
            Some(ref t) => t.to_sql(),
        }
    }
}
