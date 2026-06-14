#pragma once

#include "Overlay.hpp"
#include <QPushButton>

class UiModel;

class SessionButton : public QPushButton {
  Q_OBJECT

public:
  explicit SessionButton(const QString &text = QString(),
                         QWidget *parent = nullptr);
};

class SessionOverlay : public Overlay {
  Q_OBJECT

public:
  explicit SessionOverlay(UiModel *model);
};
