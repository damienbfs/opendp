#[cfg(feature = "ffi")]
mod ffi;

mod shr;

use crate::combinators::{OdometerAnswer, OdometerQuery};
use crate::core::{
    Domain, Function, Measure, Measurement, Metric, MetricSpace, Odometer, PrivacyMap,
    StabilityMap, Transformation,
};
use crate::error::{Error, ErrorVariant, Fallible};
use crate::interactive::{Answer, Query, Queryable};
use std::fmt::Debug;

const ERROR_URL: &str = "https://github.com/opendp/opendp/discussions/297";

macro_rules! assert_components_match {
    ($variant:ident, $v1:expr, $v2:expr) => {
        if &$v1 != &$v2 {
            return Err($crate::combinators::mismatch_error(
                $crate::error::ErrorVariant::$variant,
                &$v1,
                &$v2,
            ));
        }
    };
}
pub(crate) use assert_components_match;

pub(crate) fn mismatch_error<T: Debug>(variant: ErrorVariant, struct1: &T, struct2: &T) -> Error {
    let str1 = format!("{:?}", struct1);
    let str2 = format!("{:?}", struct2);

    let mode = match &variant {
        ErrorVariant::DomainMismatch => "domain",
        ErrorVariant::MetricMismatch => "metric",
        ErrorVariant::MeasureMismatch => "measure",
        _ => unimplemented!("unrecognized error variant"),
    };

    let explanation = if str1 == str2 {
        format!("\n    The structure of the intermediate {mode}s are the same, but the parameters differ.\n    shared_{mode}: {str1}\n", mode=mode, str1=str1)
    } else {
        format!(
            "\n    output_{mode}: {struct1}\n    input_{mode}:  {struct2}\n",
            mode = mode,
            struct1 = str1,
            struct2 = str2
        )
    };

    Error {
        variant,
        message: Some(format!(
            "Intermediate {}s don't match. See {}{}",
            mode, ERROR_URL, explanation
        )),
        backtrace: err!(@backtrace),
    }
}

/// Construct the functional composition (`measurement1` ○ `transformation0`).
/// Returns a Measurement that when invoked, computes `measurement1(transformation0(x))`.
///
/// # Arguments
/// * `measurement1` - outer measurement/mechanism
/// * `transformation0` - inner transformation
///
/// # Generics
/// * `DI` - Input Domain.
/// * `DX` - Intermediate Domain.
/// * `TO` - Output Type.
/// * `MI` - Input Metric.
/// * `MX` - Intermediate Metric.
/// * `MO` - Output Measure.
pub fn make_chain_mt<DI, DX, TO, MI, MX, MO>(
    measurement1: &Measurement<DX, TO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
) -> Fallible<Measurement<DI, TO, MI, MO>>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
{
    assert_components_match!(
        DomainMismatch,
        transformation0.output_domain,
        measurement1.input_domain
    );
    assert_components_match!(
        MetricMismatch,
        transformation0.output_metric,
        measurement1.input_metric
    );

    Measurement::new(
        transformation0.input_domain.clone(),
        Function::make_chain(&measurement1.function, &transformation0.function),
        transformation0.input_metric.clone(),
        measurement1.output_measure.clone(),
        PrivacyMap::make_chain(&measurement1.privacy_map, &transformation0.stability_map),
    )
}

/// Construct the functional composition (`odometer1` ○ `transformation0`).
/// Returns an Odometer that when invoked, computes `odometer1(transformation0(x))`.
///
/// # Arguments
/// * `odometer1` - outer odometer
/// * `transformation0` - inner transformation
///
/// # Generics
/// * `DI` - Input Domain.
/// * `DX` - Intermediate Domain.
/// * `Q` - Query Type.
/// * `A` - Answer Type.
/// * `MI` - Input Metric.
/// * `MX` - Intermediate Metric.
/// * `MO` - Output Measure.
pub fn make_chain_ot<DI, DX, Q, A, MI, MX, MO>(
    odometer1: &Odometer<
        DX,
        Queryable<OdometerQuery<Q, MX::Distance>, OdometerAnswer<A, MO::Distance>>,
        MX,
        MO,
    >,
    transformation0: &Transformation<DI, DX, MI, MX>,
) -> Fallible<
    Odometer<
        DI,
        Queryable<OdometerQuery<Q, MI::Distance>, OdometerAnswer<A, MO::Distance>>,
        MI,
        MO,
    >,
>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    Q: 'static + Clone,
    A: 'static,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
{
    assert_components_match!(
        DomainMismatch,
        transformation0.output_domain,
        odometer1.input_domain
    );
    assert_components_match!(
        MetricMismatch,
        transformation0.output_metric,
        odometer1.input_metric
    );

    let trans_function = transformation0.function.clone();
    let trans_map = transformation0.stability_map.clone();
    let odo_function = odometer1.function.clone();
    Odometer::new(
        transformation0.input_domain.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            let mut inner_qbl = odo_function.eval(&trans_function.eval(arg)?)?;
            let trans_map = trans_map.clone();

            Queryable::new(
                move |_qbl, query: Query<OdometerQuery<Q, MI::Distance>>| match query {
                    Query::External(OdometerQuery::Invoke(q)) => inner_qbl
                        .eval_invoke(q.clone())
                        .map(OdometerAnswer::Invoke)
                        .map(Answer::External),
                    Query::External(OdometerQuery::Map(d_in)) => inner_qbl
                        .eval_map(trans_map.eval(d_in)?)
                        .map(OdometerAnswer::Map)
                        .map(Answer::External),
                    Query::Internal(query) => inner_qbl.eval_query(Query::Internal(query)),
                },
            )
        }),
        transformation0.input_metric.clone(),
        odometer1.output_measure.clone(),
    )
}

/// Construct the functional composition (`transformation1` ○ `transformation0`).
/// Returns a Measurement that when invoked, computes `transformation1(transformation0(x))`.
///
/// # Arguments
/// * `transformation1` - outer transformation
/// * `transformation0` - inner transformation
///
/// # Generics
/// * `DI` - Input Domain.
/// * `DX` - Intermediate Domain.
/// * `DO` - Output Domain.
/// * `MI` - Input Metric.
/// * `MX` - Intermediate Metric.
/// * `MO` - Output Metric.
pub fn make_chain_tt<DI, DX, DO, MI, MX, MO>(
    transformation1: &Transformation<DX, DO, MX, MO>,
    transformation0: &Transformation<DI, DX, MI, MX>,
) -> Fallible<Transformation<DI, DO, MI, MO>>
where
    DI: 'static + Domain,
    DX: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MX: 'static + Metric,
    MO: 'static + Metric,
    (DI, MI): MetricSpace,
    (DX, MX): MetricSpace,
    (DO, MO): MetricSpace,
{
    assert_components_match!(
        DomainMismatch,
        transformation0.output_domain,
        transformation1.input_domain
    );

    assert_components_match!(
        MetricMismatch,
        transformation0.output_metric,
        transformation1.input_metric
    );

    Transformation::new(
        transformation0.input_domain.clone(),
        transformation1.output_domain.clone(),
        Function::make_chain(&transformation1.function, &transformation0.function),
        transformation0.input_metric.clone(),
        transformation1.output_metric.clone(),
        StabilityMap::make_chain(
            &transformation1.stability_map,
            &transformation0.stability_map,
        ),
    )
}

/// Construct the functional composition (`postprocess1` ○ `measurement0`).
/// Returns a Measurement that when invoked, computes `postprocess1(measurement0(x))`.
/// Used to represent non-interactive postprocessing.
///
/// # Arguments
/// * `postprocess1` - outer postprocessing transformation
/// * `measurement0` - inner measurement/mechanism
///
/// # Generics
/// * `DI` - Input Domain.
/// * `TX` - Intermediate Type.
/// * `TO` - Output Type.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_chain_pm<DI, TX, TO, MI, MO>(
    postprocess1: &Function<TX, TO>,
    measurement0: &Measurement<DI, TX, MI, MO>,
) -> Fallible<Measurement<DI, TO, MI, MO>>
where
    DI: 'static + Domain,
    TX: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    Measurement::new(
        measurement0.input_domain.clone(),
        Function::make_chain(postprocess1, &measurement0.function),
        measurement0.input_metric.clone(),
        measurement0.output_measure.clone(),
        measurement0.privacy_map.clone(),
    )
}

/// Construct the functional composition (`function1` ○ `odometer0`).
/// Returns an Odometer that when invoked, computes `function1(odometer0(x))`.
///
/// # Arguments
/// * `function1` - outer function
/// * `odometer1` - inner odometer
///
/// # Generics
/// * `DI` - Input Domain.
/// * `Q` - Query Type.
/// * `A1` - Initial Answer Type.
/// * `A2` - Outcome Answer Type.
/// * `MI` - Input Metric.
/// * `MO` - Output Measure.
pub fn make_chain_po<DI, Q, A1, A2, MI, MO>(
    function1: &Function<A1, A2>,
    odometer0: &Odometer<
        DI,
        Queryable<OdometerQuery<Q, MI::Distance>, OdometerAnswer<A1, MO::Distance>>,
        MI,
        MO,
    >,
) -> Fallible<
    Odometer<
        DI,
        Queryable<OdometerQuery<Q, MI::Distance>, OdometerAnswer<A2, MO::Distance>>,
        MI,
        MO,
    >,
>
where
    DI: 'static + Domain,
    Q: 'static + Clone,
    A1: 'static,
    A2: 'static,
    MI: 'static + Metric,
    MI::Distance: Clone,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    let function1 = function1.clone();
    let odometer0_function = odometer0.function.clone();
    Odometer::new(
        odometer0.input_domain.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            let mut inner_qbl = odometer0_function.eval(arg)?;
            let function1 = function1.clone();

            Queryable::new(move |_qbl, query: Query<OdometerQuery<Q, MI::Distance>>| {
                match query {
                    Query::External(OdometerQuery::Invoke(q)) => function1
                        .eval(&inner_qbl.eval_invoke(q.clone())?)
                        .map(OdometerAnswer::Invoke)
                        .map(Answer::External),
                    Query::External(OdometerQuery::Map(d_in)) => inner_qbl
                        .eval_map(d_in.clone())
                        .map(OdometerAnswer::Map)
                        .map(Answer::External),
                    Query::Internal(query) => {
                        // since A1 != A2, must repack into another Answer struct with a different type for A
                        let Answer::Internal(answer) = inner_qbl.eval_query(Query::Internal(query))?
                        else {return fallible!(FailedCast, "expected internal answer"); };

                        Ok(Answer::Internal(answer))
                    }
                }
            })
        }),
        odometer0.input_metric.clone(),
        odometer0.output_measure.clone(),
    )
}

// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::domains::AtomDomain;
    use crate::error::ExplainUnwrap;
    use crate::measures::MaxDivergence;
    use crate::metrics::AbsoluteDistance;

    use super::*;

    #[test]
    fn test_make_chain_mt() -> Fallible<()> {
        let input_domain0 = AtomDomain::<u8>::default();
        let output_domain0 = AtomDomain::<i32>::default();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = AbsoluteDistance::<i32>::default();
        let output_metric0 = AbsoluteDistance::<i32>::default();
        let stability_map0 = StabilityMap::new_from_constant(1);

        let transformation0 = Transformation::new(
            input_domain0,
            output_domain0,
            function0,
            input_metric0,
            output_metric0,
            stability_map0,
        )?;
        let input_domain1 = AtomDomain::<i32>::default();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = AbsoluteDistance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_map1 = PrivacyMap::new(|d_in: &i32| *d_in as f64 + 1.);
        let measurement1 = Measurement::new(
            input_domain1,
            function1,
            input_metric1,
            output_measure1,
            privacy_map1,
        )?;
        let chain = make_chain_mt(&measurement1, &transformation0)?;

        let arg = 99_u8;
        let ret = chain.invoke(&arg).unwrap_test();
        assert_eq!(ret, 101.0);

        let d_in = 99_i32;
        let d_out = chain.map(&d_in).unwrap_test();
        assert_eq!(d_out, 100.);

        Ok(())
    }

    #[test]
    fn test_make_chain_tt() -> Fallible<()> {
        let input_domain0 = AtomDomain::<u8>::default();
        let output_domain0 = AtomDomain::<i32>::default();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = AbsoluteDistance::<i32>::default();
        let output_metric0 = AbsoluteDistance::<i32>::default();
        let stability_map0 = StabilityMap::new_from_constant(1);
        let transformation0 = Transformation::new(
            input_domain0,
            output_domain0,
            function0,
            input_metric0,
            output_metric0,
            stability_map0,
        )?;
        let input_domain1 = AtomDomain::<i32>::default();
        let output_domain1 = AtomDomain::<f64>::default();
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let input_metric1 = AbsoluteDistance::<i32>::default();
        let output_metric1 = AbsoluteDistance::<i32>::default();
        let stability_map1 = StabilityMap::new_from_constant(1);
        let transformation1 = Transformation::new(
            input_domain1,
            output_domain1,
            function1,
            input_metric1,
            output_metric1,
            stability_map1,
        )?;
        let chain = make_chain_tt(&transformation1, &transformation0).unwrap_test();

        let arg = 99_u8;
        let ret = chain.invoke(&arg).unwrap_test();
        assert_eq!(ret, 101.0);

        let d_in = 99_i32;
        let d_out = chain.map(&d_in).unwrap_test();
        assert_eq!(d_out, 99);

        Ok(())
    }

    #[test]
    fn test_make_chain_pm() -> Fallible<()> {
        let input_domain0 = AtomDomain::<u8>::default();
        let function0 = Function::new(|a: &u8| (a + 1) as i32);
        let input_metric0 = AbsoluteDistance::<i32>::default();
        let output_measure0 = MaxDivergence::<i32>::default();
        let privacy_map0 = PrivacyMap::new_from_constant(1);
        let measurement0 = Measurement::new(
            input_domain0,
            function0,
            input_metric0,
            output_measure0,
            privacy_map0,
        )?;
        let function1 = Function::new(|a: &i32| (a + 1) as f64);
        let chain = make_chain_pm(&function1, &measurement0)?;

        let arg = 99_u8;
        let ret = chain.invoke(&arg)?;
        assert_eq!(ret, 101.0);

        let d_in = 99_i32;
        let d_out = chain.map(&d_in)?;
        assert_eq!(d_out, 99);

        Ok(())
    }
}
