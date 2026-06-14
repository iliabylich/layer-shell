#pragma once

#include "Overlay.hpp"

class QLabel;
class QTimer;
class UiModel;

class KbModOverlay : public Overlay {
  Q_OBJECT

public:
  explicit KbModOverlay(UiModel *model);

public Q_SLOTS:
  void showTemporarily(const QString &icon, const QString &text);

private:
  QLabel *icon = nullptr;
  QLabel *status = nullptr;
  QTimer *timer = nullptr;
};
