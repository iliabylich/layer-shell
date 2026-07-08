#include "openssl-wrapper.h"

int __openssl_SSL_CTX_set_min_proto_version(SSL_CTX *ctx, int version) {
  return SSL_CTX_set_min_proto_version(ctx, version);
}

long __openssl_SSL_set_tlsext_host_name(SSL *ssl, char *name) {
  return SSL_set_tlsext_host_name(ssl, name);
}
