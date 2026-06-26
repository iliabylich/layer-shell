#pragma once

#include <QList>
#include <QString>

class Config {
public:
  static QStringList getTerminalCommand(void *io);
  static QStringList getPingCommand(void *io);
  static QString getTerminalLabel(void *io);
};
