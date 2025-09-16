#define assert(cond, fmt, ...)                                                 \
  do {                                                                         \
    if (!(cond)) {                                                             \
      fprintf(stderr, "%s:%d: Assertion failed: " fmt "\n", __FILE__,          \
              __LINE__, ##__VA_ARGS__);                                        \
      abort();                                                                 \
    }                                                                          \
  } while (0)

#define checked_fmt(buf, fmt, ...)                                             \
  do {                                                                         \
    size_t len = snprintf(buf, sizeof(buf), fmt, __VA_ARGS__);                 \
    assert(len < sizeof(buf),                                                  \
           "not enough static space to format a string: %lu vs %lu", len,      \
           sizeof(buf));                                                       \
  } while (0)
