#include "Config.hpp"
#include "bindings.hpp"

QStringList
string_ref_array_to_qstring_list(IO_FFIArray<const char *> strings) {
  QStringList out;

  for (size_t i = 0; i < strings.len; i++) {
    out.push_back(QString::fromUtf8(strings.ptr[i]));
  }
  return out;
}

QStringList Config::getTerminalCommand(void *io) {
  return string_ref_array_to_qstring_list(io_get_config(io)->terminal.command);
}
QStringList Config::getPingCommand(void *io) {
  return string_ref_array_to_qstring_list(io_get_config(io)->ping);
}
QString Config::getTerminalLabel(void *io) {
  return QString::fromUtf8(io_get_config(io)->terminal.label);
}
