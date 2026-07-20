#pragma once

#include <QList>
#include <QString>

class IO;

class Config {
public:
  static QStringList getTerminalCommand(IO *io);
  static QStringList getPingCommand(IO *io);
  static QString getTerminalLabel(IO *io);
};
