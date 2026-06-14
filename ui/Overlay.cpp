#include "Overlay.hpp"
#include <LayerShellQt/Window>
#include <QBoxLayout>
#include <QFrame>
#include <QHBoxLayout>
#include <QKeyEvent>
#include <QVBoxLayout>

Overlay::Overlay(UiModel *model) : QWidget(), model(model) {
  setWindowFlag(Qt::FramelessWindowHint);
  setWindowFlag(Qt::WindowDoesNotAcceptFocus);
  setAttribute(Qt::WA_TranslucentBackground);
  setAutoFillBackground(false);
  setFocusPolicy(Qt::NoFocus);

  winId();
  QWindow *handle = windowHandle();
  if (!handle) {
    qFatal("Failed to create a native Qt window handle");
  }
  layer = LayerShellQt::Window::get(handle);
}

void Overlay::keyPressEvent(QKeyEvent *event) {
  if (close_on_escape_ && event->key() == Qt::Key_Escape) {
    hide();
    event->accept();
    return;
  }

  QWidget::keyPressEvent(event);
}

void Overlay::toggle() { setVisible(!isVisible()); }

void Overlay::setCloseOnEscape(bool close_on_escape) {
  close_on_escape_ = close_on_escape;
}

QBoxLayout *Overlay::initCenteredLayout(const QSize &size,
                                        const QString &name) {
  auto *hlayout = new QHBoxLayout(this);
  hlayout->setContentsMargins(0, 0, 0, 0);

  auto *frame = new QFrame;
  frame->setObjectName(name);
  frame->setFixedSize(size);
  hlayout->addWidget(frame, 0, Qt::AlignCenter);

  auto *vlayout = new QHBoxLayout(frame);
  vlayout->setContentsMargins(0, 0, 0, 0);
  vlayout->setSpacing(0);

  return vlayout;
}
