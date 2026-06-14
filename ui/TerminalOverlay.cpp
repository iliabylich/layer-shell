#include "TerminalOverlay.hpp"
#include "Overlay.hpp"

#include <LayerShellQt/Window>
#include <QFrame>
#include <QHBoxLayout>
#include <QKeyEvent>
#include <QVBoxLayout>
#include <QWindow>
#include <qtermwidget.h>

class TerminalWidget : public QTermWidget {
public:
  TerminalWidget(QWidget *parent, QStringList command)
      : QTermWidget(0, parent) {
    setTerminalFont(QFont("AdwaitaMono Nerd Font Mono", 12));
    setScrollBarPosition(QTermWidget::ScrollBarRight);
    setHistorySize(-1);
    setAutoClose(false);
    setColorScheme("Linux");
    setShellProgram(command.first());
    setArgs(command.sliced(1));
  }
};

//

TerminalOverlay::TerminalOverlay(UiModel *model,
                                 const QString &layer_shell_namespace,
                                 const QStringList &command)
    : Overlay(model) {
  constexpr int OverlayWidth = 1000;
  constexpr int OverlayHeight = 700;

  auto *layout =
      initCenteredLayout(QSize(OverlayWidth, OverlayHeight), "TerminalOverlay");

  layout->setContentsMargins(5, 5, 5, 5);

  term = new TerminalWidget(this, command);
  layout->addWidget(term);
  term->startShellProgram();
  connect(term, &QTermWidget::termKeyPressed, this,
          [this](QKeyEvent *event) { this->keyPressEvent(event); });

  layer->setScope(layer_shell_namespace);
  layer->setLayer(LayerShellQt::Window::LayerOverlay);
  layer->setKeyboardInteractivity(
      LayerShellQt::Window::KeyboardInteractivityExclusive);
  setCloseOnEscape(true);
}
