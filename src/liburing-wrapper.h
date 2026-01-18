#include <liburing.h>

int __liburing_queue_init(unsigned int entries, struct io_uring *ring,
                          unsigned int flags);
int __liburing_submit(struct io_uring *ring);
int __liburing_submit_and_wait(struct io_uring *ring, unsigned int wait_nr);
int __liburing_wait_cqe(struct io_uring *ring, struct io_uring_cqe **cqe_ptr);
int __liburing_wait_cqe_timeout(struct io_uring *ring,
                                struct io_uring_cqe **cqe_ptr,
                                struct __kernel_timespec *ts);
void __liburing_cqe_seen(struct io_uring *ring, struct io_uring_cqe *cqe);
void __liburing_queue_exit(struct io_uring *ring);

struct io_uring_sqe *__liburing_get_sqe(struct io_uring *ring);

void __liburing_prep_socket(struct io_uring_sqe *sqe, int domain, int type,
                            int protocol, unsigned int flags);
void __liburing_prep_connect(struct io_uring_sqe *sqe, int fd,
                             const struct sockaddr *addr, socklen_t addrlen);
void __liburing_prep_write(struct io_uring_sqe *sqe, int fd, const void *buf,
                           unsigned int nbytes, __u64 offset);
void __liburing_prep_read(struct io_uring_sqe *sqe, int fd, void *buf,
                          unsigned int nbytes, __u64 offset);
void __liburing_prep_close(struct io_uring_sqe *sqe, int fd);
void __liburing_prep_openat(struct io_uring_sqe *sqe, int dfd, const char *path,
                            int flags, mode_t mode);
