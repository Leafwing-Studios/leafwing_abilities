//! Pools are a reservoir of resources that can be used to pay for abilities, or keep track of character state.
//!
//! Unlike charges, pools are typically shared across abilities.
//!
//! Life, mana, energy and rage might all be modelled effectively as pools.
//! Pools have a maximum value, have a minimum value of zero, can regenerate over time, and can be spent to pay for abilities.

use core::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use crate::CannotUseAbility;

/// A reservoir of a resource that can be used to pay for abilities, or keep track of character state.
///
/// Each type that implements this trait should be stored on a component (or, if your actions are globally unique, a resource),
/// and contains information about the current, max
pub trait Pool:
    Add<Self>
    + AddAssign<Self>
    + Sub<Self>
    + SubAssign<Self>
    + PartialEq
    + PartialOrd
    + Sized
    + Mul<f32>
    + Div<f32>
{
    /// The minimum value of the pool type.
    ///
    /// At this point, no resources remain to be spent.
    const ZERO: Self;

    /// The current quantity of resources in the pool.
    fn current(&self) -> Self;

    /// The maximum quantity of resources that this pool of resources can store.
    fn max(&self) -> Self;

    /// The amount of resources recovered in one second.
    ///
    /// This value may be negative, in the case of automatically decaying pools (like rage).
    fn regen_per_second(&self) -> Self;

    /// Spend the specified amount from the pool, if there is that much available.
    ///
    /// Otherwise, return the error [`CannotUseAbility::PoolEmpty`].
    fn expend(&mut self, amount: Self) -> Result<(), CannotUseAbility>;

    /// Replenish the pool by the specified amount.
    ///
    /// This cannot cause the pool to exceed maximum value that can be stored in the pool.
    fn replenish(&mut self, amount: Self);
}
