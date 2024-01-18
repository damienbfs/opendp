use std::fmt::Debug;

use dashu::{
    float::round::{
        mode::{Down, Up},
        ErrorBounds,
    },
    integer::UBig,
};

#[cfg(test)]
mod test;

mod gumbel;
pub use gumbel::GumbelPSRN;

mod tulap;
pub use tulap::TulapPSRN;

mod uniform;
pub use uniform::UniformPSRN;

use crate::error::Fallible;

pub trait PSRN {
    type Edge: PartialOrd + Debug;
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
            match self.lower().zip(other.upper()) {
                Some((l, r)) if l > r => break true,
                _ => (),
            }
            match self.upper().zip(other.lower()) {
                Some((l, r)) if l < r => break false,
                _ => (),
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
    psrn: &mut TI,
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
