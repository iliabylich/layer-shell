#pragma once

#include "Overlay.hpp"

class UiModel;

class TopBar : public Overlay {
  Q_OBJECT

public:
  explicit TopBar(UiModel *model);

Q_SIGNALS:
  void weatherClicked();
  void terminalClicked();
  void pingClicked();
  void powerClicked();

private:
  UiModel *model;
};
