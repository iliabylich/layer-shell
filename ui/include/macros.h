#define NULL_TERMINATED_STRING_LIST(...)                                       \
  (const char *[]) { __VA_ARGS__, NULL }

#define CSS NULL_TERMINATED_STRING_LIST

#define VTE_CMD NULL_TERMINATED_STRING_LIST
