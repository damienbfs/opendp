use std::clone::Clone;
use std::convert::TryFrom;
use std::iter::IntoIterator;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Mul;

use rug::{Integer, Rational};

const GRID_SIZE:usize = 10;
const DENOM:usize = 1000000000;

/// Privacy Loss Distribution from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf

/// A privacy loss value (log-likelihood)
#[derive(Clone, PartialEq)]
pub struct PLDistribution {
    /// We represent PLD as p_y/p_x -> p_x for p_y <= p_x
    /// contrary to http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf (p_x/p_y -> p_x)
    exp_privacy_loss_probabilities: BTreeMap<Rational, Rational>// Change that to have only p_y_x<1 TODO
}

impl<'a> PLDistribution {
    pub fn new<I>(exp_privacy_loss_probabilitiies:I) -> PLDistribution
    where I: IntoIterator<Item=(Rational, Rational)> {
        let mut p_y_x_p_x:BTreeMap<Rational, Rational> = BTreeMap::new();
        let mut sum_p = Rational::from(0);
        for (p_y_x, p_x) in exp_privacy_loss_probabilitiies {
            if &Rational::from(0) <= &p_y_x && &p_y_x <= &Rational::from(1) && &Rational::from(0) < &p_x {
                p_y_x_p_x.entry(p_y_x.clone())
                    .and_modify(|p| *p += p_x.clone() )
                    .or_insert(p_x.clone());
                // Compute the total sum of probabilities (count half of p_y_x==1)
                if &p_y_x != &Rational::from(1) {sum_p += (Rational::from(1)+p_y_x)*p_x;} else {sum_p += p_x;}
            }
        }
        for (_, p_x) in p_y_x_p_x.iter_mut() {
            *p_x /= &sum_p;
        }
        // Add 0 and 1 entries
        p_y_x_p_x.entry(Rational::from(0))
            .and_modify(|p| *p += Rational::from(0) )
            .or_insert(Rational::from(0));
        p_y_x_p_x.entry(Rational::from(1))
            .and_modify(|p| *p += Rational::from(0) )
            .or_insert(Rational::from(0));
        PLDistribution {exp_privacy_loss_probabilities:p_y_x_p_x}
    }

    /// Use the formula from http://proceedings.mlr.press/v108/koskela20b/koskela20b.pdf
    pub fn delta(&self, exp_epsilon: &Rational) -> Rational {
        if exp_epsilon==&Rational::from(0) {
            return Rational::from(1);
        }
        let mut result = Rational::from(0);
        for (p_y_x,p_x) in self.exp_privacy_loss_probabilities.iter() {
            if p_y_x<&exp_epsilon.clone().recip() {
                result += (Rational::from(1)-p_y_x*exp_epsilon.clone())*p_x;
            }
            // Avoid the double count of p_y_x==1
            if p_y_x>exp_epsilon && p_y_x!=&Rational::from(1) {
                result += (p_y_x-exp_epsilon.clone())*p_x;
            }
        }
        result
    }

    /// Compute a delta and simplifies it to the simple fraction just below
    /// The degree of simplification is expressed by giving the target denominator
    pub fn simplified_delta(&self, exp_epsilon: Rational, denom: usize) -> Rational {
        let delta = self.delta(&exp_epsilon);
        let num = Integer::from(denom) * delta.numer() / delta.denom();
        Rational::from((num,denom))
    }

    /// Compute the alphas and the betas
    pub fn tradeoff(&self) -> Vec<(Rational, Rational)> {
        let mut result = Vec::new();
        let mut exp_epsilons_set:BTreeSet<Rational> = BTreeSet::new();
        // Initialize the set of possible exp_eps
        for exp_epsilon in self.exp_privacy_loss_probabilities.keys() {
            exp_epsilons_set.insert(exp_epsilon.clone());
            if exp_epsilon>&Rational::from(0) {
                exp_epsilons_set.insert(exp_epsilon.clone().recip());
            }
        }
        // Reverse the exp epsilons to have them by decreasing order
        let exp_epsilons: Vec<Rational> = exp_epsilons_set.into_iter().rev().collect();
        // Insert the first points
        result.push((Rational::from(0), Rational::from(1)));
        let mut last_exp_epsilon = exp_epsilons[0].clone();
        let mut last_delta= self.delta(&last_exp_epsilon);
        result.push((Rational::from(0), Rational::from(1)-&last_delta));
        for i in 1..exp_epsilons.len() {
            let exp_epsilon = exp_epsilons[i].clone();
            let delta = self.delta(&exp_epsilon);
            let denom = exp_epsilon.clone()-&last_exp_epsilon;
            result.push((
                (last_delta.clone()-&delta)/&denom,
                ((Rational::from(1)-&last_delta)*&exp_epsilon-(Rational::from(1)-&delta)*&last_exp_epsilon)/&denom,
            ));
            last_exp_epsilon = exp_epsilon.clone();
            last_delta = delta.clone();
        }
        result.push((Rational::from(1), Rational::from(0)));
        result
    }

    pub fn f(&self) -> Vec<(f64, f64)> {
        self.tradeoff().into_iter().map(|(a,b)| (a.to_f64(), b.to_f64())).collect()
    }

    /// Compute a vector of deltas
    pub fn deltas(&self, exp_epsilons:Vec<Rational>) -> Vec<(Rational, Rational)> {
        exp_epsilons.into_iter().map(|e| {(e.clone(),self.delta(&e))}).collect()
    }

    pub fn simplified(&self) -> PLDistribution {
        let mut result_exp_epsilons: Vec<Rational> = Vec::new();
        let mut result_alpha_betas: Vec<(Rational,Rational)> = Vec::new();
        let mut result_exp_privacy_loss_probabilities: Vec<(Rational,Rational)> = Vec::new();
        // Select a set of exp_epsilons
        for i in 0..GRID_SIZE {
            result_exp_epsilons.push(Rational::from(GRID_SIZE+1)/Rational::from(i+1));
        }
        // Compute alphas and betas
        result_alpha_betas.push((Rational::from(0), Rational::from(1)));
        let mut last_exp_epsilon = result_exp_epsilons[0].clone();
        let mut last_delta= self.simplified_delta(last_exp_epsilon.clone(), DENOM);
        result_alpha_betas.push((Rational::from(0), Rational::from(1)-&last_delta));
        for i in 1..result_exp_epsilons.len() {
            let exp_epsilon = result_exp_epsilons[i].clone();
            let delta = self.simplified_delta(exp_epsilon.clone(), DENOM);
            let denom = exp_epsilon.clone()-&last_exp_epsilon;
            result_alpha_betas.push((
                (last_delta.clone()-&delta)/&denom,
                ((Rational::from(1)-&last_delta)*&exp_epsilon-(Rational::from(1)-&delta)*&last_exp_epsilon)/&denom,
            ));
            last_exp_epsilon = exp_epsilon.clone();
            last_delta = delta.clone();
        }
        println!("{:?}", result_alpha_betas.iter().map(|(a,b)| (a.to_f64(), b.to_f64())).collect::<Vec<(f64,f64)>>());
        // Compute probabilities
        let mut last_alpha = result_alpha_betas[0].0.clone();
        let mut last_beta= result_alpha_betas[0].1.clone();
        for i in 1..result_alpha_betas.len() {
            let alpha = result_alpha_betas[i].0.clone();
            let beta= result_alpha_betas[i].1.clone();
            if &beta != &last_beta {
                let exp_epsilon = -(alpha.clone()-&last_alpha)/(beta.clone()-&last_beta);
                result_exp_privacy_loss_probabilities.push((exp_epsilon, -(beta.clone()-&last_beta)));
            }
            last_alpha = alpha.clone();
            last_beta = beta.clone();
        }
        println!("{:?}", result_exp_privacy_loss_probabilities.iter().map(|(l,p)| (l.to_f64(), p.to_f64())).collect::<Vec<(f64,f64)>>());
        PLDistribution::new(result_exp_privacy_loss_probabilities)
    }
}

/// Compute the composition of PLDs
impl Mul for &PLDistribution {
    type Output = PLDistribution;
    fn mul(self, other: &PLDistribution) -> PLDistribution {
        let mut result = PLDistribution {exp_privacy_loss_probabilities:BTreeMap::new()};
        for (s_p_y_x,s_p_x) in &self.exp_privacy_loss_probabilities {
            for (o_p_y_x,o_p_x) in &other.exp_privacy_loss_probabilities {
                // s,o
                let so_p_y_x = s_p_y_x.clone() * o_p_y_x;
                let so_p_x = s_p_x.clone() * o_p_x;
                result.exp_privacy_loss_probabilities.entry(so_p_y_x)
                    .and_modify(|p_x| { *p_x += so_p_x.clone() })
                    .or_insert(so_p_x);
                // s_inv,o
                if s_p_y_x>&Rational::from(0) && s_p_y_x<&Rational::from(1) {
                    let so_p_y_x = s_p_y_x.clone().recip() * o_p_y_x;
                    let so_p_x = s_p_x.clone()*s_p_y_x * o_p_x;
                    if so_p_y_x<=Rational::from(1) {
                        result.exp_privacy_loss_probabilities.entry(so_p_y_x)
                            .and_modify(|p_x| { *p_x += so_p_x.clone() })
                            .or_insert(so_p_x);
                    }
                }
                // s,o_inv
                if o_p_y_x>&Rational::from(0) && o_p_y_x<&Rational::from(1) {
                    let so_p_y_x = o_p_y_x.clone().recip() * s_p_y_x;
                    let so_p_x = s_p_x.clone() * o_p_x * o_p_y_x;
                    if so_p_y_x<=Rational::from(1) {
                        result.exp_privacy_loss_probabilities.entry(so_p_y_x)
                            .and_modify(|p_x| { *p_x += so_p_x.clone() })
                            .or_insert(so_p_x);
                    }
                }
            }
        }
        result
    }
}

impl Default for PLDistribution {
    fn default() -> Self {
        PLDistribution::new([(Rational::from(1),Rational::from(1))])
    }
}

impl<Q> From<Vec<(Q,Q)>> for PLDistribution
where Rational: TryFrom<Q> {
    fn from(exp_privacy_loss_probabilities: Vec<(Q,Q)>) -> PLDistribution {
        let rational_exp_privacy_loss_probabilities: Vec<(Rational,Rational)> = exp_privacy_loss_probabilities.into_iter().map(|(epl, p)| 
            (Rational::try_from(epl).unwrap_or_default(), Rational::try_from(p).unwrap_or_default())
        ).collect();
        PLDistribution::new(rational_exp_privacy_loss_probabilities)
    }
}