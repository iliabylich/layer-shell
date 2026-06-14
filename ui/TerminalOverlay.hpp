#pragma once

#include "Overlay.hpp"

class TerminalWidget;

class TerminalOverlay : public Overlay {
  Q_OBJECT

public:
  TerminalOverlay(UiModel *model, const QString &layer_shell_namespace,
                  const QStringList &command);

private:
  TerminalWidget *term = nullptr;
};
