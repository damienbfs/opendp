use std::{collections::BTreeSet, sync::Arc};

use polars_plan::{
    dsl::{Expr, Operator},
    plans::DslPlan,
    prelude::GroupbyOptions,
    utils::expr_output_name,
};

use crate::{measurements::expr_noise::NoiseArgs, polars::match_plugin};

use super::Fallible;

pub(crate) struct MatchGroupBy {
    pub input: DslPlan,
    pub keys: Vec<Expr>,
    pub aggs: Vec<Expr>,
    pub threshold: Option<(String, u32)>,
}

pub(crate) fn match_group_by(mut plan: DslPlan) -> Fallible<Option<MatchGroupBy>> {
    let threshold = if let DslPlan::Filter { input, predicate } = plan {
        plan = input.as_ref().clone();
        Some(is_threshold_predicate(predicate)?)
    } else {
        None
    };

    let DslPlan::GroupBy {
        input,
        keys,
        aggs,
        apply,
        maintain_order,
        options,
    } = plan
    else {
        return Ok(None);
    };

    if options.as_ref() != &GroupbyOptions::default() {
        return fallible!(MakeMeasurement, "Unsupported options in logical plan. Do not optimize the lazyframe passed into the constructor. Options should be default, but are {:?}", options);
    }

    if apply.is_some() {
        return fallible!(MakeMeasurement, "Apply is not supported in logical plan");
    }

    if maintain_order {
        return fallible!(MakeMeasurement, "The order of keys is sensitive");
    }

    Ok(Some(MatchGroupBy {
        input: Arc::unwrap_or_clone(input),
        keys,
        aggs,
        threshold,
    }))
}

pub fn match_grouping_columns(keys: Vec<Expr>) -> Fallible<BTreeSet<String>> {
    Ok(keys
        .iter()
        .map(|e| {
            Ok(match e {
                Expr::Column(name) => vec![(*name).to_string()],
                Expr::Columns(names) => names.iter().map(|s| s.to_string()).collect(),
                e => {
                    return fallible!(
                        MakeMeasurement,
                        "Expected column expression in keys, found {:?}",
                        e
                    )
                }
            })
        })
        .collect::<Fallible<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}

pub(super) fn find_len_expr(
    exprs: &Vec<Expr>,
    name: Option<&str>,
) -> Fallible<(String, NoiseArgs)> {
    // only keep expressions that compute the length
    (exprs.iter())
        .find_map(|e| is_len_expr(e, name))
        .ok_or_else(|| {
            err!(
                MakeMeasurement,
                "expected exactly one DP length expression with name: {:?}",
                name
            )
        })
}

fn is_len_expr(expr: &Expr, name: Option<&str>) -> Option<(String, NoiseArgs)> {
    let output_name = expr_output_name(expr).ok()?;

    // check if the expression matches the expected name
    if let Some(name) = name {
        if name != output_name.as_ref() {
            return None;
        }
    }
    // remove any aliasing in the expression
    let expr = expr.clone().meta().undo_aliases();

    let (inputs, args) = match_plugin(&expr, "noise").ok().flatten()?;

    let [input] = <&[Expr; 1]>::try_from(inputs.as_slice()).ok()?;

    if input != &Expr::Len {
        return None;
    }

    Some((output_name.to_string(), args))
}

pub(crate) fn is_threshold_predicate(expr: Expr) -> Fallible<(String, u32)> {
    let Expr::BinaryExpr { left, op, right } = expr else {
        return fallible!(MakeMeasurement, "threshold must be a binary expression");
    };

    use Operator::{Gt, Lt};

    let (name, value) = match (left.as_ref(), op, right.as_ref()) {
        (Expr::Column(name), Gt, Expr::Literal(value)) => (name, value),
        (Expr::Literal(value), Lt, Expr::Column(name)) => (name, value),
        _ => {
            return fallible!(
                MakeMeasurement,
                "expected an expression of the form 'col(name) > threshold'"
            )
        }
    };

    Ok((
        name.to_string(),
        value.to_any_value().unwrap().extract().unwrap(),
    ))
}
