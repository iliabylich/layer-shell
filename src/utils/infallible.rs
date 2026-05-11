use crate::{
    modules::FallibleModule,
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};

pub(crate) struct InfallibleModule<M> {
    module: Option<M>,
}

impl<M: FallibleModule> InfallibleModule<M> {
    const NAME: &str = M::MODULE_ID.as_str();

    pub(crate) const fn new(module: M) -> Self {
        Self {
            module: Some(module),
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.module.as_mut()?.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<M::Output> {
        match self.module.as_mut()?.try_satisfy(satisfy, res) {
            Ok(output) => output,
            Err(err) => {
                log::error!(target: Self::NAME, "crash, stopping. satisfy={satisfy:?}, res={res}, err: {err:?}");
                self.module = None;
                None
            }
        }
    }

    pub(crate) const fn inner(&mut self) -> Option<&mut M> {
        self.module.as_mut()
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        let Some(module) = self.module.as_mut() else {
            return;
        };

        if let Err(err) = module.try_tick(tick) {
            log::error!(target: Self::NAME, "crash, stopping. err: {err:?}");
            self.module = None;
        }
    }

    #[expect(clippy::unused_self)]
    pub(crate) const fn module_id(&self) -> ModuleId {
        M::MODULE_ID
    }
}
