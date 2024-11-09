use anyhow::{anyhow, Result};
use hawk_parser::Expr;
use ion_rs::{
    element::{Element, Value},
    types::{Int, Str, Struct},
    IonData,
};
use std::borrow::Cow;

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

pub fn resolve_cond(item: &Struct, expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Equal(lhs, rhs) => {
            let lhs = resolve_expr(item, lhs)?;
            let rhs = resolve_expr(item, rhs)?;
            if lhs.ion_type() != rhs.ion_type() {
                let lhs = match lhs.as_ref() {
                    Value::String(text) => text.as_ref().to_owned(),
                    _ => lhs.as_ref().to_string(),
                };
                let rhs = match rhs.as_ref() {
                    Value::String(text) => text.as_ref().to_owned(),
                    _ => rhs.as_ref().to_string(),
                };
                Ok(Value::Bool(lhs == rhs))
            } else {
                Ok(Value::Bool(lhs == rhs))
            }
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
