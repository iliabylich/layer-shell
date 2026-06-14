#pragma once

#include <QList>
#include <QString>

class Config {
public:
  static QStringList getTerminalCommand();
  static QStringList getPingCommand();
  static QString getTerminalLabel();
};
