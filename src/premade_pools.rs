//! Convenient premade resource [`Pool`] types to get you started.
//!
//! These can be annoying due to orphan rules that prevent you from implementing your own methods,
//! so feel free to copy-paste them (without attribution) into your own source to make new variants.

use crate::pool::{MaxPoolLessThanMin, Pool};
use bevy::prelude::{Component, Resource};
use core::fmt::{Display, Formatter};
use core::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use derive_more::{Add, AddAssign, Sub, SubAssign};

/// A premade resource pool for life (aka health, hit points or HP).
pub mod life {
    use bevy::reflect::Reflect;

    use crate::pool::RegeneratingPool;

    use super::*;

    /// The amount of life available to a unit.
    /// If they lose it all, they die or pass out.
    ///
    /// This is intended to be stored as a component on each entity.
    #[derive(Debug, Clone, PartialEq, Component, Resource, Reflect)]
    pub struct LifePool {
        /// The current life.
        current: Life,
        /// The maximum life that can be stored.
        max: Life,
        /// The amount of life regenerated per second.
        pub regen_per_second: Life,
    }

    impl LifePool {
        /// Creates a new [`LifePool`] with the supplied settings.
        ///
        /// # Panics
        /// Panics if `current` is greater than `max`.
        /// Panics if `current` or max is negative.

        pub fn new(current: Life, max: Life, regen_per_second: Life) -> Self {
            assert!(current <= max);
            assert!(current >= LifePool::MIN);
            assert!(max >= LifePool::MIN);
            Self {
                current,
                max,
                regen_per_second,
            }
        }
    }

    /// A quantity of life, used to modify a [`LifePool`].
    ///
    /// This can be used for damage computations, life regeneration, healing and so on.
    #[derive(
        Debug, Clone, Copy, PartialEq, PartialOrd, Default, Add, Sub, AddAssign, SubAssign, Reflect,
    )]
    pub struct Life(pub f32);

    impl Mul<f32> for Life {
        type Output = Life;

        fn mul(self, rhs: f32) -> Life {
            Life(self.0 * rhs)
        }
    }

    impl Mul<Life> for f32 {
        type Output = Life;

        fn mul(self, rhs: Life) -> Life {
            Life(self * rhs.0)
        }
    }

    impl Div<f32> for Life {
        type Output = Life;

        fn div(self, rhs: f32) -> Life {
            Life(self.0 / rhs)
        }
    }

    impl Div<Life> for Life {
        type Output = f32;

        fn div(self, rhs: Life) -> f32 {
            self.0 / rhs.0
        }
    }

    impl Pool for LifePool {
        type Quantity = Life;
        const MIN: Life = Life(0.);

        fn current(&self) -> Self::Quantity {
            self.current
        }

        fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
            let actual_value = Life(new_quantity.0.clamp(0., self.max.0));
            self.current = actual_value;
            self.current
        }

        fn max(&self) -> Self::Quantity {
            self.max
        }

        fn set_max(&mut self, new_max: Self::Quantity) -> Result<(), MaxPoolLessThanMin> {
            if new_max < Self::MIN {
                Err(MaxPoolLessThanMin)
            } else {
                self.max = new_max;
                self.set_current(self.current);
                Ok(())
            }
        }
    }

    impl RegeneratingPool for LifePool {
        fn regen_per_second(&self) -> Self::Quantity {
            self.regen_per_second
        }

        fn set_regen_per_second(&mut self, new_regen_per_second: Self::Quantity) {
            self.regen_per_second = new_regen_per_second;
        }
    }

    impl Add<Life> for LifePool {
        type Output = Self;

        fn add(mut self, rhs: Life) -> Self::Output {
            self.set_current(self.current + rhs);
            self
        }
    }

    impl Sub<Life> for LifePool {
        type Output = Self;

        fn sub(mut self, rhs: Life) -> Self::Output {
            self.set_current(self.current - rhs);
            self
        }
    }

    impl AddAssign<Life> for LifePool {
        fn add_assign(&mut self, rhs: Life) {
            self.set_current(self.current + rhs);
        }
    }

    impl SubAssign<Life> for LifePool {
        fn sub_assign(&mut self, rhs: Life) {
            self.set_current(self.current - rhs);
        }
    }

    impl Display for Life {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Display for LifePool {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}/{}", self.current, self.max)
        }
    }
}

/// A premade resource pool for mana (aka MP).
pub mod mana {
    use bevy::reflect::Reflect;

    use crate::pool::RegeneratingPool;

    use super::*;

    /// The amount of mana available to a unit.
    /// Units must spend mana to cast spells according to their [`AbilityCosts<A, Mana>`](crate::pool::AbilityCosts) component.
    ///
    /// This is intended to be stored as a component on each entity.
    #[derive(Debug, Clone, PartialEq, Component, Resource, Reflect)]
    pub struct ManaPool {
        /// The current mana.
        current: Mana,
        /// The maximum mana that can be stored.
        max: Mana,
        /// The amount of mana regenerated per second.
        pub regen_per_second: Mana,
    }

    impl ManaPool {
        /// Creates a new [`ManaPool`] with the supplied settings.
        ///
        /// # Panics
        /// Panics if `current` is greater than `max`.
        /// Panics if `current` or `max` is negative.
        pub fn new(current: Mana, max: Mana, regen_per_second: Mana) -> Self {
            assert!(current <= max);
            assert!(current >= ManaPool::MIN);
            assert!(max >= ManaPool::MIN);
            Self {
                current,
                max,
                regen_per_second,
            }
        }
    }

    /// A quantity of mana, used to modify a [`ManaPool`].
    ///
    /// This can be used for ability costs, mana regeneration and so on.
    #[derive(
        Debug, Clone, Copy, PartialEq, PartialOrd, Default, Add, Sub, AddAssign, SubAssign, Reflect,
    )]
    pub struct Mana(pub f32);

    impl Mul<f32> for Mana {
        type Output = Mana;

        fn mul(self, rhs: f32) -> Mana {
            Mana(self.0 * rhs)
        }
    }

    impl Mul<Mana> for f32 {
        type Output = Mana;

        fn mul(self, rhs: Mana) -> Mana {
            Mana(self * rhs.0)
        }
    }

    impl Div<f32> for Mana {
        type Output = Mana;

        fn div(self, rhs: f32) -> Mana {
            Mana(self.0 / rhs)
        }
    }

    impl Div<Mana> for Mana {
        type Output = f32;

        fn div(self, rhs: Mana) -> f32 {
            self.0 / rhs.0
        }
    }

    impl Pool for ManaPool {
        type Quantity = Mana;
        const MIN: Mana = Mana(0.);

        fn current(&self) -> Self::Quantity {
            self.current
        }

        fn set_current(&mut self, new_quantity: Self::Quantity) -> Self::Quantity {
            let actual_value = Mana(new_quantity.0.clamp(0., self.max.0));
            self.current = actual_value;
            self.current
        }

        fn max(&self) -> Self::Quantity {
            self.max
        }

        fn set_max(&mut self, new_max: Self::Quantity) -> Result<(), MaxPoolLessThanMin> {
            if new_max < Self::MIN {
                Err(MaxPoolLessThanMin)
            } else {
                self.max = new_max;
                self.set_current(self.current);
                Ok(())
            }
        }
    }

    impl RegeneratingPool for ManaPool {
        fn regen_per_second(&self) -> Self::Quantity {
            self.regen_per_second
        }

        fn set_regen_per_second(&mut self, new_regen_per_second: Self::Quantity) {
            self.regen_per_second = new_regen_per_second;
        }
    }

    impl Add<Mana> for ManaPool {
        type Output = Self;

        fn add(mut self, rhs: Mana) -> Self::Output {
            self.set_current(self.current + rhs);
            self
        }
    }

    impl Sub<Mana> for ManaPool {
        type Output = Self;

        fn sub(mut self, rhs: Mana) -> Self::Output {
            self.set_current(self.current - rhs);
            self
        }
    }

    impl AddAssign<Mana> for ManaPool {
        fn add_assign(&mut self, rhs: Mana) {
            self.set_current(self.current + rhs);
        }
    }

    impl SubAssign<Mana> for ManaPool {
        fn sub_assign(&mut self, rhs: Mana) {
            self.set_current(self.current - rhs);
        }
    }

    impl Display for Mana {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Display for ManaPool {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}/{}", self.current, self.max)
        }
    }
}
