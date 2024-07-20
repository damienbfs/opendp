// TODO:
// . Mike TODO at some point: remove SMDCurve in favor of function (separate)

// . implement β(α) curve conversion -> draft
// . implement posterior curve -> done
// . relative risk curve -> done
// . FFI for above
// . visualizations in Python
// . adjust internal implementation to use δ(ε) curve, add ε(δ) helper via bs
// . parameterize gaussian mechanism
// . combinator for making a trivial δ(ε) from (ε, δ) (already have a conversion from ε, to (ε, 0)) 
use crate::{
    core::Function,
    error::Fallible,
    measures::SMDCurve
};



impl SMDCurve<f64> {
    /// Find beta at a given `alpha`.
    pub fn beta(&self, alpha: f64) -> Fallible<f64> {
        find_best_supporting_beta(&self, alpha)
    }

    /// Create an α(β) curve supported on num_approximations supporting functions.
    pub fn tradeoff(&self, num_approximations: Option<u32>) -> Fallible<Function<f64, f64>> {
        profile_to_tradeoff(self.clone(), num_approximations)
    }
}


fn profile_to_tradeoff(
    curve: SMDCurve<f64>,
    num_approximations: Option<u32>,
) -> Fallible<Function<f64, f64>> {

    let num_approximations = num_approximations.unwrap_or(100);
    let alphas: Vec<f64> = (0..=num_approximations).map(|i| i as f64 / num_approximations as f64).collect();
    let mut betas = Vec::new();

    for alpha in alphas {
        let beta = find_best_supporting_beta(&curve, alpha).unwrap(); // Will this panic if beta is an error?
        betas.push(beta);
    
        //if beta > alpha {
            // TODO Not sure we want to do a symmetry here
            // If tradeoff curve is very "bad", ie. close to the lower left
            // corner, we will end up with very few approximations.
        //}
    }

    Ok(Function::new_fallible(move |alpha: &f64| -> Fallible<f64> {
        if *alpha < 0.0 || *alpha > 1.0 {
            return fallible!(FailedMap, "alpha must be in [0, 1]");
        }

        let index_left = (*alpha * num_approximations as f64).floor(); // index is closest approx of alpha * num_approximations
        let alpha_left = index_left / num_approximations as f64;

        if alpha_left == *alpha {
            return Ok(betas[index_left as usize]);
        }

        let beta_left = betas[index_left as usize];
        let beta_right = betas[(index_left as usize) + 1];
        let beta = (beta_left + beta_right) / 2.0;

        Ok(beta)
    }))
}

/// Finds the best supporting tradeoff curve and returns the highest
/// beta for a given a privacy curve and alpha
///
/// # Arguments:
/// * `curve` - Privacy curve
/// * `alpha` - must be within [0, 1]
fn find_best_supporting_beta(curve: &SMDCurve<f64>, alpha: f64) -> Fallible<f64> {
    // Ternary search for delta that maximizes beta in the interval [0, 1]
    // Could be improved with golden search algorithm or setting
    // delta_mid_left to (delta_right - delta_left)/2 - very_small_value
    let mut delta_left = 0.0;
    let mut delta_right = 1.0;
    loop {
        let third = (delta_right - delta_left) / 3.0;
        let delta_mid_left = delta_left + third;
        let delta_mid_right = delta_right - third;
        
        let epsilon_mid_left = curve.epsilon(&delta_mid_left)?;
        let epsilon_mid_right = curve.epsilon(&delta_mid_right)?;
        let beta_mid_left = support_tradeoff(alpha, epsilon_mid_left, delta_mid_left);
        let beta_mid_right = support_tradeoff(alpha, epsilon_mid_right, delta_mid_right);

        if beta_mid_left > beta_mid_right {
            delta_right = delta_mid_right;
        } else if beta_mid_left < beta_mid_right {
            delta_left = delta_mid_left;
        } else { // beta_mid_left == beta_mid_right
            delta_left = delta_mid_left;
            delta_right = delta_mid_right;
        }

        // Stoping criteria
        if delta_right - delta_left < 0.000001 { // TODO arbitrary -> can also select deltas directly and precompute?
            let delta = (delta_right - delta_left) / 2.0;
            let epsilon = curve.epsilon(&delta)?;
            let beta = support_tradeoff(alpha, epsilon, delta);
            return Ok(beta);
        }
    }
}


/// Computes the β parameter associated with an (ε, δ) linear supporting curve at α
/// 
/// # Arguments
/// * `alpha`- must be within [0, 1]
/// * `epsilon`- must be non-negative
/// * `delta`- must be within [0, 1]
fn support_tradeoff(alpha: f64, epsilon: f64, delta: f64) -> f64 {
    let left = 1.0 - delta - (epsilon.exp() * alpha);
    let right = (-epsilon).exp() * (1.0 - delta - alpha);

    left.max(right).max(0.0)
}


/// Computes the posterior curve given tradeoff curve and attacker's prior probability
/// in a membership attack.
/// 
/// The returned Function takes an alpha value and returns the attacker's posterior.
/// TODO does the posterior only take values in (0, 1] instead of [0, 1]?
/// 
/// # Arguments
/// * `tradeoff_curve` - Tradeoff curve for the measurement
/// * `prior` - Attacker's prior probability.
pub fn get_posterior_curve(tradeoff_curve: Function<f64, f64>, prior: f64) -> Fallible<Function<f64, f64>> {
    Ok(Function::new_fallible(move |alpha: &f64| {
        let beta = tradeoff_curve.eval(alpha).unwrap();
        Ok((prior * (1.0 - beta)) / ((1.0 - prior)* (*alpha) + prior * (1.0 - beta)))
    }))
}


/// Computes the relative risk curve given tradeoff curve and attacker's prior probability
/// in a membership attack.
/// 
/// The returned Function takes an alpha value and returns the relative risk.
/// TODO does the relative risk only take values in (0, 1] instead of [0, 1]?
/// 
/// # Arguments
/// * `tradeoff_curve` - Tradeoff curve for the measurement
/// * `prior` - Attacker's prior probability.
pub fn get_relative_risk_curve(tradeoff_curve: Function<f64, f64>, prior: f64) -> Fallible<Function<f64, f64>> {
    
    Ok(Function::new_fallible(move |alpha: &f64| {
        let beta = tradeoff_curve.eval(alpha).unwrap();
        Ok((1.0 - beta) / ((1.0 - prior)* (*alpha) + prior * (1.0 - beta)))
    }))
}


#[test]
fn test_all() -> Fallible<()> {
    let epsilon: f64 = 1.0;
    let pure_dp_privacy_profile = move |delta: f64| {
        if delta == 0.0 {
            return Ok(epsilon);
        }
        
        Ok((epsilon.exp() - delta).ln())
    };
    let smd_curve = SMDCurve::new(move |&delta| pure_dp_privacy_profile(delta));
    
    // Tradeoff
    let tradeoff_curve = profile_to_tradeoff(smd_curve, None).unwrap();
    println!("tradeoff(0) = {}", tradeoff_curve.eval(&0.0).unwrap());
    println!("tradeoff(0.27) = {}", tradeoff_curve.eval(&0.27).unwrap());
    println!("tradeoff(1) = {}", tradeoff_curve.eval(&1.0).unwrap());
    
    // Posterior
    let posterior_curve: Function<f64, f64> = get_posterior_curve(tradeoff_curve.clone(), 0.5).unwrap();
    println!("posterior(prior=0.5, alpha=0) = {}", posterior_curve.eval(&0.0).unwrap());
    println!("posterior(prior=0.5, alpha=0.5) = {}", posterior_curve.eval(&0.5).unwrap());
    println!("posterior(prior=0.5, alpha=1) = {}", posterior_curve.eval(&0.5).unwrap());

    // Relative risk
    
    let relative_risk_curve: Function<f64, f64> = get_relative_risk_curve(tradeoff_curve.clone(), 0.5).unwrap();
    println!("relative_risk(prior=0.5, alpha=0) = {}", relative_risk_curve.eval(&0.0).unwrap());
    println!("relative_risk(prior=0.5, alpha=0.5) = {}", relative_risk_curve.eval(&0.5).unwrap());
    println!("relative_risk(prior=0.5, alpha=1) = {}", relative_risk_curve.eval(&0.5).unwrap());

    let alphas: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
    for a in &alphas {
        println!("{}", relative_risk_curve.eval(&a).unwrap());
    }


    /*
    Questions:
    - Rust closure "move"
    - Why need to clone tradeoff curve, isn't it copied with the "move" keyword? -> do the clone
    - what delta "precision" is enough, arbitrary choice, leave option to user?

    TODOs:
    - arbitrary choice of "delta precision"
    - replace ternary search? seems relatively quick as-is.
    - tests
    - function names, etc..
    */

    Ok(())

}