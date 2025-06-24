mod emitter;
mod service;
mod service_ref;

pub use emitter::Emitter;
pub use service::Service;
pub use service_ref::ServiceRef;

#[macro_export]
macro_rules! service {
    ($name:ident, $event:ty, $loop:expr) => {
        pub struct $name {
            service: $crate::ServiceRef<$event>,
        }

        impl $name {
            pub fn start() -> Self {
                Self {
                    service: $crate::Service::start($loop),
                }
            }

            pub async fn stop(self) -> Result<()> {
                self.service.stop().await
            }

            pub async fn recv(&mut self) -> Option<Event> {
                self.service.recv().await
            }
        }
    };
}
