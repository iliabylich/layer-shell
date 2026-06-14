#include "Config.hpp"
#include "bindings.hpp"

QStringList string_ref_array_to_qstring_list(const char **items) {
  QStringList out;
  if (!items) {
    return out;
  }

  for (const char **item = items; *item; item++) {
    out.push_back(QString::fromUtf8(*item));
  }
  return out;
}

QStringList Config::getTerminalCommand() {
  return string_ref_array_to_qstring_list(io_get_config()->terminal.command);
}
QStringList Config::getPingCommand() {
  return string_ref_array_to_qstring_list(io_get_config()->ping);
}
QString Config::getTerminalLabel() {
  return QString::fromUtf8(io_get_config()->terminal.label);
}
