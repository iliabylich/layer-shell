#pragma once

#include "Overlay.hpp"

class UiModel;
class IO_IO;

class TopBar : public Overlay {
  Q_OBJECT

public:
  explicit TopBar(UiModel *model, IO_IO *io);

Q_SIGNALS:
  void weatherClicked();
  void terminalClicked();
  void pingClicked();
  void powerClicked();

private:
  UiModel *model;
};
