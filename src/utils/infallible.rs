use crate::{
    modules::FallibleModule,
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};

pub(crate) struct InfallibleModule<M> {
    module: Option<M>,
}

impl<M> InfallibleModule<M>
where
    M: FallibleModule,
{
    const NAME: &str = M::MODULE_ID.as_str();

    pub(crate) const fn new(module: M) -> Self {
        Self {
            module: Some(module),
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.module.as_mut()?.wants() {
            Ok(output) => output,
            Err(err) => {
                log::error!(target: Self::NAME, ".wants() returned an err, stopping. err: {err:?}");
                self.module = None;
                None
            }
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<M::Output> {
        match self.module.as_mut()?.try_satisfy(satisfy, res) {
            Ok(output) => output,
            Err(err) => {
                log::error!(target: Self::NAME, ".satisfy() returned an errir, stopping. satisfy={satisfy:?}, res={res}, err: {err:?}");
                self.module = None;
                None
            }
        }
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

impl<M> std::ops::Deref for InfallibleModule<M> {
    type Target = Option<M>;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<M> std::ops::DerefMut for InfallibleModule<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
    }
}
