use dashu::{
    float::{
        round::{
            mode::{Down, Up},
            ErrorBounds,
        },
        FBig,
    },
    integer::UBig,
};

mod gumbel;
pub use gumbel::GumbelPSRN;

mod laplace;
pub use laplace::LaplacePSRN;

mod uniform;
pub use uniform::UniformPSRN;

use crate::{error::Fallible, traits::RoundCast};

pub trait PSRN {
    type Edge: PartialOrd;
    fn edge<R: ODPRound>(&self) -> Option<Self::Edge>;
    fn refine(&mut self) -> Fallible<()>;
    fn refinements(&self) -> usize;

    fn lower(&self) -> Option<Self::Edge> {
        self.edge::<Down>()
    }
    fn upper(&self) -> Option<Self::Edge> {
        self.edge::<Up>()
    }

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals are disjoint.
    fn greater_than(&mut self, other: &mut Self) -> Fallible<bool> {
        Ok(loop {
            if self.lower() > other.upper() {
                break true;
            }
            if self.upper() < other.lower() {
                break false;
            }
            if self.refinements() < other.refinements() {
                self.refine()?
            } else {
                other.refine()?
            }
        })
    }
}

pub trait ODPRound: ErrorBounds {
    const UBIG: UBig;
    type Complement: ODPRound<Complement = Self>;
}

impl ODPRound for Down {
    const UBIG: UBig = UBig::ZERO;
    type Complement = Up;
}

impl ODPRound for Up {
    const UBIG: UBig = UBig::ONE;
    type Complement = Down;
}

/// Check if `psrn` is greater than `threshold`
pub fn check_above<RV: PSRN>(psrn: &mut RV, threshold: &RV::Edge) -> Fallible<bool> {
    loop {
        if psrn.lower().as_ref() > Some(threshold) {
            return Ok(true);
        }
        if psrn.upper().as_ref() < Some(threshold) {
            return Ok(false);
        }
        psrn.refine()?;
    }
}

/// Refine `psrn` until both bounds of interval round to same TO
pub fn pinpoint<TI: PSRN<Edge = FBig>, TO: RoundCast<FBig> + PartialEq>(
    psrn: &mut LaplacePSRN,
) -> Fallible<TO> {
    loop {
        psrn.refine()?;
        let Some((l, r)) = psrn.lower().zip(psrn.upper()) else {
            continue;
        };
        let (l, r) = (TO::round_cast(l)?, TO::round_cast(r)?);
        if l == r {
            return Ok(l);
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn test_progression<RV: PSRN>(
        sampler: &mut RV,
        min_refinements: usize,
    ) -> (RV::Edge, RV::Edge)
    where
        RV::Edge: PartialOrd,
    {
        loop {
            sampler.refine().unwrap();
            let Some((l, r)) = sampler.lower().zip(sampler.upper()) else {
                continue;
            };
            assert!(l <= r);

            if sampler.refinements() >= min_refinements {
                return (l, r);
            }
        }
    }
}
