use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use hawk_parser::Expr;
use ion_rs::{
    element::{Element, Value},
    external::bigdecimal::{num_bigint::BigInt, BigDecimal},
    types::{Decimal, Int, IonType, Str, Struct, Timestamp},
    IonData,
};
use std::{borrow::Cow, str::FromStr};

pub mod csv;

pub trait IonIterator: Iterator<Item = Element> {}

pub fn resolve_var<'a>(item: &'a Struct, expr: &Expr) -> Result<&'a Value> {
    if let Expr::Variable(variable) = expr {
        if variable.starts_with('$') {
            if let Some(field_number) = variable.strip_prefix('$') {
                let field_number: usize = field_number.parse()?;
                if field_number >= 1 && field_number <= item.len() {
                    if let Some((_, element)) = item.fields().nth(field_number - 1) {
                        return Ok(element.value());
                    }
                }
            }
        }
    }
    Err(anyhow!("No value"))
}

pub struct ValueImplicitConversion {}
impl ValueImplicitConversion {
    fn coerce_value(value: &Value, ion_type: IonType) -> Result<Cow<Value>> {
        match (value, ion_type) {
            (Value::String(v), IonType::Bool) => Ok(Cow::Owned(Value::Bool(v.text().parse()?))),
            (Value::String(v), IonType::Int) => {
                Ok(Cow::Owned(Value::Int(Int::BigInt(v.text().parse()?))))
            }
            (Value::String(v), IonType::Float) => Ok(Cow::Owned(Value::Float(v.text().parse()?))),
            (Value::String(v), IonType::Decimal) => {
                let decimal = BigDecimal::from_str(v.text())?;
                Ok(Cow::Owned(Value::Decimal(Decimal::from(decimal))))
            }
            (Value::String(v), IonType::Timestamp) => {
                let datetime: DateTime<FixedOffset> = v.text().parse()?;
                Ok(Cow::Owned(Value::Timestamp(Timestamp::from(datetime))))
            }
            (Value::Int(Int::I64(v)), IonType::Float) => Ok(Cow::Owned(Value::Float(*v as f64))),
            (Value::Int(Int::I64(v)), IonType::Decimal) => {
                let decimal = BigDecimal::new(BigInt::from(*v), 0);
                Ok(Cow::Owned(Value::Decimal(Decimal::from(decimal))))
            }
            (Value::Int(Int::BigInt(v)), IonType::Decimal) => {
                let decimal = BigDecimal::new(v.clone(), 0);
                Ok(Cow::Owned(Value::Decimal(Decimal::from(decimal))))
            }
            _ => Ok(Cow::Borrowed(value)),
        }
    }

    fn coerce<'a, 'b>(lhs: &'a Value, rhs: &'b Value) -> Result<(Cow<'a, Value>, Cow<'b, Value>)> {
        if lhs.ion_type() != rhs.ion_type() {
            Ok((
                ValueImplicitConversion::coerce_value(lhs, rhs.ion_type())?,
                ValueImplicitConversion::coerce_value(rhs, lhs.ion_type())?,
            ))
        } else {
            Ok((Cow::Borrowed(lhs), Cow::Borrowed(rhs)))
        }
    }
}

pub fn resolve_cond(item: &Struct, expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Equal(lhs, rhs) => {
            let (lhs, rhs) = (resolve_expr(item, lhs)?, resolve_expr(item, rhs)?);
            let (lhs, rhs) = ValueImplicitConversion::coerce(lhs.as_ref(), rhs.as_ref())?;
            Ok(Value::Bool(lhs == rhs))
        }
        Expr::NotEqual(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            Ok(Value::Bool(lhs != rhs))
        }
        Expr::LessThan(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            Ok(Value::from(
                IonData::from(lhs.as_ref()) < IonData::from(rhs.as_ref()),
            ))
        }
        Expr::LessThanOrEqual(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            Ok(Value::from(
                IonData::from(lhs.as_ref()) <= IonData::from(rhs.as_ref()),
            ))
        }
        Expr::GreaterThan(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            Ok(Value::from(
                IonData::from(lhs.as_ref()) > IonData::from(rhs.as_ref()),
            ))
        }
        Expr::GreaterThanOrEqual(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            Ok(Value::from(
                IonData::from(lhs.as_ref()) >= IonData::from(rhs.as_ref()),
            ))
        }
        Expr::And(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            match (lhs.as_ref(), rhs.as_ref()) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(*lhs && *rhs)),
                _ => Err(anyhow!("Error value!")),
            }
        }
        Expr::Or(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            match (lhs.as_ref(), rhs.as_ref()) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(*lhs || *rhs)),
                _ => Err(anyhow!("Error value!")),
            }
        }
        _ => Err(anyhow!("No value")),
    }
}

pub fn resolve_expr<'a>(item: &'a Struct, expr: &Expr) -> Result<Cow<'a, Value>> {
    match expr {
        Expr::Variable(_) => Ok(Cow::Borrowed(resolve_var(item, expr)?)),
        Expr::Integer(v) => Ok(Cow::Owned(Value::Int(Int::I64(*v)))),
        Expr::String(v) => Ok(Cow::Owned(Value::String(Str::from(v.to_owned())))),
        _ => Ok(Cow::Owned(resolve_cond(item, expr)?)),
    }
}
