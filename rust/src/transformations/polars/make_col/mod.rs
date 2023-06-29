use num::One;
use polars::prelude::*;
use std::collections::BTreeSet;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    Context, ExprDomain, ExprMetric, LazyFrameContext, LazyFrameDomain, LazyGroupByContext,
};
use crate::error::*;
use crate::traits::DistanceConstant;

pub trait GroupingColumns {
    fn grouping_columns(&self) -> Vec<String>;
}

impl GroupingColumns for LazyFrameContext {
    fn grouping_columns(&self) -> Vec<String> {
        vec![]
    }
}

impl GroupingColumns for LazyGroupByContext {
    fn grouping_columns(&self) -> Vec<String> {
        self.columns.clone()
    }
}

/// Make a Transformation that returns a col(<column_name>) expression for a Lazy Frame.
///
/// # Generics
/// * `M` - Dataset Metric type.
///
pub fn make_col<M, C: Context>(
    input_domain: ExprDomain<C>,
    input_metric: M::OuterMetric,
    col_name: &str,
) -> Fallible<Transformation<ExprDomain<C>, ExprDomain<C>, M::OuterMetric, M::OuterMetric>>
where
    M: ExprMetric<C>,
    M::Distance: DistanceConstant<M::Distance> + One + Clone,
    C: GroupingColumns,
    (ExprDomain<C>, M::OuterMetric): MetricSpace,
{
    if input_domain.lazy_frame_domain.column(col_name).is_none() {
        return fallible!(MakeTransformation, "{} is not in dataframe", col_name);
    }

    let context_columns = input_domain.context.grouping_columns();

    let mut columns_to_keep = BTreeSet::from_iter(context_columns);
    columns_to_keep.insert(String::from(col_name));

    let series_domains = input_domain
        .clone()
        .lazy_frame_domain
        .series_domains
        .into_iter()
        .filter(|s| columns_to_keep.contains(&s.field.name.to_string()))
        .collect();

    let margins = input_domain
        .clone()
        .lazy_frame_domain
        .margins
        .into_iter()
        .filter(|(s, _)| s.is_subset(&columns_to_keep))
        .collect();

    let output_domain = ExprDomain {
        lazy_frame_domain: LazyFrameDomain {
            series_domains,
            margins,
        },
        context: input_domain.context.clone(),
        active_column: Some(col_name.to_string()),
    };

    let column_name = col_name.to_string();

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(
            move |(frame, expr): &(C::Value, Expr)| -> Fallible<(C::Value, Expr)> {
                if expr != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "make_col has to be the first expression in the expression chain"
                    );
                }
                Ok((frame.clone(), col(&*column_name)))
            },
        ),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(M::Distance::one()),
    )
}

#[cfg(test)]
mod test_make_col {
    use crate::{
        domains::{AtomDomain, SeriesDomain},
        metrics::SymmetricDistance,
    };

    use super::*;

    fn get_test_data() -> (ExprDomain<LazyGroupByContext>, LazyGroupBy) {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])
        .unwrap_test()
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]].unwrap_test().lazy())
        .unwrap_test()
        .with_counts(
            df!["B" => [1.0, 2.0], "count" => [2, 1]]
                .unwrap_test()
                .lazy(),
        )
        .unwrap_test()
        .with_counts(
            df!["C" => [8, 9, 10], "count" => [1, 1, 1]]
                .unwrap_test()
                .lazy(),
        )
        .unwrap_test();

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyGroupByContext {
                columns: vec![String::from("A")],
            },
            active_column: None,
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],
            "C" => &[8, 9, 10],)
        .unwrap_test()
        .lazy();

        (expr_domain, lazy_frame.groupby([col("A")]))
    }

    #[test]
    fn test_make_col_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data();
        let selected_col = "B";
        let transformation =
            make_col::<SymmetricDistance, _>(expr_domain, Default::default(), selected_col)
                .unwrap_test();

        let expr_res = transformation.invoke(&(lazy_frame, all())).unwrap_test().1;
        let expr_exp = col(selected_col);

        assert_eq!(expr_res, expr_exp);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_domain() -> Fallible<()> {
        let (expr_domain, _) = get_test_data();
        let expr_domain_context_exp = expr_domain.context.clone();

        let selected_col = "B";
        let transformation =
            make_col::<SymmetricDistance, _>(expr_domain, Default::default(), selected_col)
                .unwrap_test();

        let expr_domain_res = transformation.output_domain.clone();

        let lf_domain_exp = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])
        .unwrap_test()
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]].unwrap_test().lazy())
        .unwrap_test()
        .with_counts(
            df!["B" => [1.0, 2.0], "count" => [2, 1]]
                .unwrap_test()
                .lazy(),
        )
        .unwrap_test();

        let expr_domain_exp = ExprDomain {
            lazy_frame_domain: lf_domain_exp,
            context: expr_domain_context_exp,
            active_column: Some(selected_col.to_string()),
        };

        assert_eq!(expr_domain_res, expr_domain_exp);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_no_wildcard() -> Fallible<()> {
        let (expr_domain, lazy_groupby) = get_test_data();
        let selected_col = "B";

        let transformation =
            make_col::<SymmetricDistance, _>(expr_domain, Default::default(), selected_col)
                .unwrap_test();
        let error_res = transformation
            .invoke(&(lazy_groupby, col(selected_col)))
            .map(|v| v.1)
            .unwrap_err()
            .variant;
        let expected_error_kind = ErrorVariant::FailedFunction;

        assert_eq!(error_res, expected_error_kind);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_wrong_col() -> Fallible<()> {
        let (expr_domain, _) = get_test_data();
        let selected_col = "D";

        let error_res =
            make_col::<SymmetricDistance, _>(expr_domain, Default::default(), selected_col)
                .map(|v| v.input_domain.clone())
                .unwrap_err()
                .variant;
        let expected_error_kind = ErrorVariant::MakeTransformation;

        assert_eq!(error_res, expected_error_kind);

        Ok(())
    }
}
