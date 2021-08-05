mod map;
pub use map::*;
mod avg_four;
pub use avg_four::avg_four;

pub trait Cost {
    fn cost(&self) -> Option<f64>;
}

macro_rules! nz_cost_impls {
    ($($t:ident),*) => {
        $(
            impl Cost for Option<std::num::$t> {
                fn cost(&self) -> Option<f64> {
                    self.map(|c| c.get() as f64)
                }
            }
        )*
    };
}

nz_cost_impls!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroUsize);

impl Cost for f64 {
    fn cost(&self) -> Option<f64> {
        self.is_finite().then(|| *self)
    }
}

impl Cost for f32 {
    fn cost(&self) -> Option<f64> {
        self.is_finite().then(|| *self as f64)
    }
}
