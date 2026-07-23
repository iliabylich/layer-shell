use crate::module_id::ModuleId;
use core::assert_matches;
use rustix::{
    event::epoll,
    fd::{AsFd, BorrowedFd, OwnedFd},
};

pub struct Epoll {
    fd: OwnedFd,
}

impl Epoll {
    pub(crate) const MODULES_COUNT: usize = ModuleId::MODULES_COUNT;

    pub(crate) fn new() -> Self {
        log::trace!("Creating Epoll");
        let fd = epoll::create(epoll::CreateFlags::CLOEXEC).unwrap_or_else(|err| {
            panic!("failed to epoll_create(): {err:?}");
        });
        Self { fd }
    }

    pub(crate) fn wait_readable(&self) {
        use rustix::event::{PollFd, PollFlags, poll};

        let mut pollfds = [PollFd::new(&self, PollFlags::IN)];
        let res = poll(&mut pollfds, None);
        assert_matches!(res, Ok(_), "poll failed: {res:?}");

        let revents = pollfds[0].revents();
        assert_matches!(revents, PollFlags::IN, "poll(epollfd) returned an error");
    }

    pub(crate) fn add(&self, source: impl AsFd, module_id: ModuleId) {
        let res = epoll::add(
            &self.fd,
            source,
            epoll::EventData::new_u64(module_id as u64),
            epoll::EventFlags::IN,
        );
        assert_matches!(res, Ok(()), "failed to epoll_add()");
    }

    pub(crate) fn delete(&self, source: impl AsFd) {
        let res = epoll::delete(&self.fd, source);
        assert_matches!(res, Ok(()), "failed to epoll_add(): {res:?}");
    }

    pub(crate) fn wait(&self) -> [ModulePoll; Self::MODULES_COUNT] {
        let mut event_list = [epoll::Event {
            flags: epoll::EventFlags::empty(),
            data: epoll::EventData::new_u64(0),
        }; Self::MODULES_COUNT];

        let len = epoll::wait(
            &self.fd,
            &mut event_list,
            Some(&rustix::fs::Timespec {
                tv_sec: 0,
                tv_nsec: 0,
            }),
        )
        .unwrap_or_else(|err| panic!("failed to epoll_wait(): {err:?}"));

        let mut out = [ModulePoll::None; _];

        for (event, out) in event_list.iter().take(len).zip(out.iter_mut()) {
            let flags = event.flags;
            let data = event.data.u64();
            let is_err = flags.intersects(epoll::EventFlags::HUP | epoll::EventFlags::ERR);
            let is_readable = flags.contains(epoll::EventFlags::IN);

            let module_id = ModuleId::new(data);

            if is_err {
                *out = ModulePoll::Err(module_id);
            } else if is_readable {
                *out = ModulePoll::Readable(module_id);
            } else {
                *out = ModulePoll::None;
            }
        }

        out
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ModulePoll {
    Err(ModuleId),
    Readable(ModuleId),
    None,
}

impl AsFd for Epoll {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}
