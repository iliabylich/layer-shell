#include "liburing-wrapper.h"
#include <liburing.h>

int __liburing_queue_init(unsigned int entries, struct io_uring *ring,
                          unsigned int flags) {
  return io_uring_queue_init(entries, ring, flags);
}
int __liburing_submit(struct io_uring *ring) { return io_uring_submit(ring); }
int __liburing_submit_and_wait(struct io_uring *ring, unsigned int wait_nr) {
  return io_uring_submit_and_wait(ring, wait_nr);
}
int __liburing_wait_cqe(struct io_uring *ring, struct io_uring_cqe **cqe_ptr) {
  return io_uring_wait_cqe(ring, cqe_ptr);
}
int __liburing_wait_cqe_timeout(struct io_uring *ring,
                                struct io_uring_cqe **cqe_ptr,
                                struct __kernel_timespec *ts) {
  return io_uring_wait_cqe_timeout(ring, cqe_ptr, ts);
}
void __liburing_cqe_seen(struct io_uring *ring, struct io_uring_cqe *cqe) {
  io_uring_cqe_seen(ring, cqe);
}
void __liburing_queue_exit(struct io_uring *ring) { io_uring_queue_exit(ring); }

struct io_uring_sqe *__liburing_get_sqe(struct io_uring *ring) {
  return io_uring_get_sqe(ring);
}

void __liburing_prep_socket(struct io_uring_sqe *sqe, int domain, int type,
                            int protocol, unsigned int flags) {
  io_uring_prep_socket(sqe, domain, type, protocol, flags);
}
void __liburing_prep_connect(struct io_uring_sqe *sqe, int fd,
                             const struct sockaddr *addr, socklen_t addrlen) {
  io_uring_prep_connect(sqe, fd, addr, addrlen);
}
void __liburing_prep_write(struct io_uring_sqe *sqe, int fd, const void *buf,
                           unsigned int nbytes, __u64 offset) {
  io_uring_prep_write(sqe, fd, buf, nbytes, offset);
}
void __liburing_prep_read(struct io_uring_sqe *sqe, int fd, void *buf,
                          unsigned int nbytes, __u64 offset) {
  io_uring_prep_read(sqe, fd, buf, nbytes, offset);
}
void __liburing_prep_close(struct io_uring_sqe *sqe, int fd) {
  io_uring_prep_close(sqe, fd);
}
void __liburing_prep_openat(struct io_uring_sqe *sqe, int dfd, const char *path,
                            int flags, mode_t mode) {
  io_uring_prep_openat(sqe, dfd, path, flags, mode);
}
