use crate::{
    event_queue::EventQueue,
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use std::assert_matches;

pub(crate) trait TryWantsTrySatisfy {
    const ID: ModuleId;

    type Output: Default;

    fn try_wants(&mut self) -> Result<Option<Wants>>;
    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<Self::Output>;
}

pub(crate) trait CanStop: Sized {
    fn stopped(&mut self) -> Self;
}

pub(crate) trait WantsSatisfy: TryWantsTrySatisfy + Sized + CanStop {
    fn wants(&mut self) -> Option<Wants> {
        let wants = wants_once(self)?;
        log::trace!(target: Self::ID.as_str(), "{wants:?}");
        assert_matches!(wants_once(self), None);
        Some(wants)
    }

    fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Self::Output {
        match self.try_satisfy(satisfy, events) {
            Ok(output) => output,
            Err(err) => {
                log::error!("Module {:?} has crashed: {err:?}", Self::ID);
                *self = self.stopped();
                return Default::default();
            }
        }
    }
}

impl<T> WantsSatisfy for T where T: TryWantsTrySatisfy + Sized + CanStop {}

fn wants_once<T>(module: &mut T) -> Option<Wants>
where
    T: WantsSatisfy + Sized,
{
    match module.try_wants() {
        Ok(wants) => wants,
        Err(err) => {
            log::error!("Module {:?} has crashed: {err:?}", T::ID);
            module.stopped();
            return None;
        }
    }
}
