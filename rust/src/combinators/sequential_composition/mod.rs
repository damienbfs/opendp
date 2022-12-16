use std::any::Any;

use crate::{
    core::{Domain, Function, Measurement, Metric, PrivacyMap},
    domains::{AllDomain, QueryableDomain},
    error::Fallible,
    interactive::{ChildChange, Queryable, QueryableBase},
    traits::TotalOrd,
};

use super::BasicCompositionMeasure;

pub fn make_sequential_composition<
    DI: Domain + 'static,
    DQ: Domain + 'static,
    DA: Domain + 'static,
    MI: Metric + Default + 'static,
    MO: BasicCompositionMeasure + 'static,
>(
    input_domain: DI,
    query_domain: DQ,
    answer_domain: DA,
    output_measure: MO,
    d_in: MI::Distance,
    mut d_mids: Vec<MO::Distance>,
) -> Fallible<
    Measurement<DI, AllDomain<Measurement<DI, DQ, DA, MI, MO>>, QueryableDomain<DQ, DA>, MI, MO>,
>
where
    MI::Distance: 'static + TotalOrd + Copy,
    DI::Carrier: 'static + Clone,
    MO::Distance: 'static + TotalOrd + Copy,
{
    if d_mids.len() == 0 {
        return fallible!(MakeMeasurement, "must be at least one d_out");
    }

    // we'll iteratively pop from the end
    d_mids.reverse();

    let d_out = output_measure.compose(d_mids.clone())?;

    Ok(Measurement::new(
        input_domain,
        AllDomain::new(),
        QueryableDomain::new(query_domain, answer_domain),
        Function::new(enclose!(d_in, move |arg: &DI::Carrier| {
            // a new copy of the state variables is made each time the Function is called:

            // IMMUTABLE STATE VARIABLES
            let arg = arg.clone();

            // MUTABLE STATE VARIABLES
            let mut d_mids = d_mids.clone();

            // below, the queryable closure's arguments are
            // 1. a reference to itself (which it can use to set context)
            // 2. the query, which is a dynamically typed `&dyn Any`

            // arg, d_mids, d_in and d_out are all moved into (or captured by) the Queryable closure here
            Queryable::new(move |_self: &QueryableBase, query: &dyn Any| {
                // evaluate the measurement query and return the answer.
                //     the downcast ref attempts to downcast the &dyn Any to a specific concrete type
                //     if the query passed in was this type of measurement, the downcast will succeed
                if let Some(measurement) = query.downcast_ref::<Measurement<DI, DQ, DA, MI, MO>>() {
                    // retrieve the last distance from d_mids, or bubble an error if d_mids is empty
                    let d_mid =
                        (d_mids.last()).ok_or_else(|| err!(FailedFunction, "out of queries"))?;

                    // check that the query doesn't consume too much privacy
                    if !measurement.check(&d_in, d_mid)? {
                        return fallible!(FailedFunction, "insufficient budget for query");
                    }

                    // evaluate the query!
                    let answer = measurement.invoke(&arg)?;

                    // we've now consumed the last d_mid. This is our only state modification
                    d_mids.pop();

                    // if the answer is a queryable,
                    // wrap it so that when the child gets a query it sends a ChildChange query to this parent queryable
                    // it gives this sequential composition queryable (or any parent of this queryable)
                    // a chance to deny the child permission to execute
                    // DO::map_queryable(|queryable: QueryableBase| {
                    //     Queryable::new(move |self_child: &QueryableBase, query: &dyn Any| {
                    //         unimplemented!()
                    //     })
                    // });

                    // The box allows the return value to be dynamically typed, just like query was.
                    // Necessary because different queries have different return types.
                    // All responses are of type `Fallible<Box<dyn Any>>`
                    return Ok(Box::new(answer));
                }

                // update state based on child change
                if let Some(change) = query.downcast_ref::<ChildChange<MO::Distance>>() {
                    if change.id != d_mids.len() {
                        return fallible!(
                            FailedFunction,
                            "sequential compositor has received a new query"
                        );
                    }
                    // state won't change in response to child,
                    // but return an Ok to approve the change
                    return Ok(Box::new(()));
                }

                fallible!(FailedFunction, "unrecognized query!")
            })
        })),
        MI::default(),
        MO::default(),
        PrivacyMap::new_fallible(move |d_in_p: &MI::Distance| {
            if d_in_p.total_gt(&d_in)? {
                fallible!(
                    RelationDebug,
                    "input distance must not be greater than d_in"
                )
            } else {
                Ok(d_out.clone())
            }
        }),
    ))
}

#[cfg(test)]
mod test {

    use crate::{
        domains::{AllDomain, PolyDomain},
        measurements::make_randomized_response_bool,
        measures::MaxDivergence,
    };

    use super::*;

    #[test]
    fn test_sequential_composition() -> Fallible<()> {
        // construct sequential compositor IM
        let root = make_sequential_composition(
            AllDomain::new(),
            PolyDomain::new(),
            PolyDomain::new(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1, 0.3, 0.5],
        )?;

        // pass dataset in and receive a queryable
        let mut queryable = root.invoke(&true)?;

        let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
        let rr_query = make_randomized_response_bool(0.5, false)?;

        // pass queries into the SC queryable
        let _answer1: bool = queryable.eval(&rr_poly_query)?.get_poly()?;
        let _answer2: bool = queryable.eval(&rr_poly_query)?.get_poly()?;

        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are in AllDomain<bool>
        let sc_query_3 = make_sequential_composition(
            AllDomain::<bool>::new(),
            AllDomain::<()>::new(),
            AllDomain::<bool>::new(),
            MaxDivergence::default(),
            1,
            vec![0.1, 0.1],
        )?
        .into_poly();

        let mut answer3: Queryable<_, Queryable<(), bool>> = queryable.eval_poly(&sc_query_3)?;
        let _answer3_1: bool = answer3.eval(&rr_query)?.get()?;
        let _answer3_2: bool = answer3.eval(&rr_query)?.get()?;

        // pass a sequential composition compositor into the original SC compositor
        // This compositor expects all outputs are in PolyDomain
        let sc_query_4 = make_sequential_composition(
            AllDomain::<bool>::new(),
            PolyDomain::new(),
            PolyDomain::new(),
            MaxDivergence::default(),
            1,
            vec![0.2, 0.3],
        )?
        .into_poly();

        let mut answer4: Queryable<_, Queryable<Box<dyn Any>, Box<dyn Any>>> =
            queryable.eval_poly(&sc_query_4)?;
        let _answer4_1: bool = answer4.eval(&rr_poly_query)?.get_poly()?;
        let _answer4_2: bool = answer4.eval(&rr_poly_query)?.get_poly()?;

        Ok(())
    }
}
