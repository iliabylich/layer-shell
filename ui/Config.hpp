#pragma once

#include <QList>
#include <QString>

class IO_IO;

class Config {
public:
  static QStringList getTerminalCommand(IO_IO *io);
  static QStringList getPingCommand(IO_IO *io);
  static QString getTerminalLabel(IO_IO *io);
};
