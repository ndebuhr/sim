use std::{cell::RefCell, rc::Rc};

pub trait SimulationRng: std::fmt::Debug + rand_core::RngCore {}
impl<T: std::fmt::Debug + rand_core::RngCore> SimulationRng for T {}
pub type DynRng = Rc<RefCell<dyn SimulationRng>>;

pub(crate) fn default_rng() -> DynRng {
    Rc::new(RefCell::new(rand_pcg::Pcg64Mcg::new(42)))
}

pub fn some_dyn_rng<Rng: SimulationRng + 'static>(rng: Rng) -> Option<DynRng> {
    Some(Rc::new(RefCell::new(rng)))
}
