#include "Config.hpp"
#include "bindings.hpp"

QStringList command_to_exec_to_string_list(CommandToExec cmd) {
  QStringList out;

  for (size_t i = 0; i < cmd.len; i++) {
    out.push_back(QString::fromUtf8(cmd.ptr[i]));
  }
  return out;
}

QStringList Config::getTerminalCommand(IO *io) {
  return command_to_exec_to_string_list(io_get_terminal_cmd(io));
}
QStringList Config::getPingCommand(IO *io) {
  return command_to_exec_to_string_list(io_get_ping_cmd(io));
}
QString Config::getTerminalLabel(IO *io) {
  return QString::fromUtf8(io_get_terminal_label(io));
}
