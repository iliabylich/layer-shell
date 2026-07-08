#include <openssl/bio.h>
#include <openssl/ssl.h>
#include <openssl/tls1.h>
#include <openssl/x509v3.h>

int __openssl_SSL_CTX_set_min_proto_version(SSL_CTX *ctx, int version);
long __openssl_SSL_set_tlsext_host_name(SSL *ssl, char *name);
