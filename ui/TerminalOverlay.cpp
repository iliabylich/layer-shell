#include "TerminalOverlay.hpp"
#include "Overlay.hpp"

#include <LayerShellQt/Window>
#include <QFrame>
#include <QHBoxLayout>
#include <QKeyEvent>
#include <QShowEvent>
#include <QTimer>
#include <QVBoxLayout>
#include <QWindow>
#include <qtermwidget.h>

class TerminalWidget : public QTermWidget {
public:
  TerminalWidget(QWidget *parent, QStringList command)
      : QTermWidget(0, parent) {
    setTerminalFont(terminalFont());
    setHistorySize(-1);
    setAutoClose(false);
    setColorScheme("WhiteOnBlack");
    setScrollBarPosition(QTermWidget::NoScrollBar);
    setShellProgram(command.first());
    setArgs(command.sliced(1));
  }

private:
  static QFont terminalFont() {
    return QFont("AdwaitaMono Nerd Font Mono", 12);
  }

  void showEvent(QShowEvent *event) override {
    QTermWidget::showEvent(event);
    QTimer::singleShot(0, this, [this] { setTerminalFont(terminalFont()); });
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
