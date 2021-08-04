pub trait NodeBase {
    fn make_source(&mut self);
    fn expand(&mut self);
    fn get_expansions(&self) -> usize;
}

pub trait QueueLocation {
    fn get_location(&self) -> usize;
    fn set_location(&mut self, index: usize);
}

pub trait Parent<V> {
    fn set_parent(&mut self, p: V);
    fn get_parent(&self) -> Option<V>;
}

pub trait GValue {
    fn get_g(&self) -> f64;
    fn set_g(&mut self, g: f64);
}

pub trait LowerBound {
    fn get_lb(&self) -> f64;
    fn set_lb(&mut self, lb: f64);
}

pub trait Destination<V> {
    fn destination(&self) -> V;
}

pub trait Cost {
    fn cost(&self) -> f64;
}
