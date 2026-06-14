#include "SessionOverlay.hpp"
#include "UiModel.hpp"

#include <LayerShellQt/Window>
#include <QKeyEvent>
#include <QPushButton>
#include <QVBoxLayout>

SessionButton::SessionButton(const QString &text, QWidget *parent)
    : QPushButton(text, parent) {
  setCursor(Qt::PointingHandCursor);
}

//

SessionOverlay::SessionOverlay(UiModel *model) : Overlay(model) {
  setObjectName("SessionOverlay");

  auto *layout = new QVBoxLayout(this);
  layout->setAlignment(Qt::AlignCenter);
  layout->setSpacing(20);

  auto *lock = new SessionButton("Lock");
  layout->addWidget(lock, 0, Qt::AlignHCenter);
  connect(lock, &QPushButton::clicked, this, [this] {
    hide();
    this->model->lock();
  });

  auto *reboot = new SessionButton("Reboot");
  layout->addWidget(reboot, 0, Qt::AlignHCenter);
  connect(reboot, &QPushButton::clicked, this, [this] {
    hide();
    this->model->reboot();
  });

  auto *shutdown = new SessionButton("Shutdown");
  layout->addWidget(shutdown, 0, Qt::AlignHCenter);
  connect(shutdown, &QPushButton::clicked, this, [this] {
    hide();
    this->model->shutdown();
  });

  auto *logout = new SessionButton("Logout");
  layout->addWidget(logout, 0, Qt::AlignHCenter);
  connect(logout, &QPushButton::clicked, this, [this] {
    hide();
    this->model->logout();
  });

  layer->setScope("LayerShell/SessionScreen");
  layer->setLayer(LayerShellQt::Window::LayerOverlay);
  layer->setAnchors(LayerShellQt::Window::Anchors(
      LayerShellQt::Window::AnchorTop | LayerShellQt::Window::AnchorBottom |
      LayerShellQt::Window::AnchorLeft | LayerShellQt::Window::AnchorRight));
  layer->setKeyboardInteractivity(
      LayerShellQt::Window::KeyboardInteractivityExclusive);
  setCloseOnEscape(true);

  QObject::connect(model, &UiModel::sessionToggleRequested, this,
                   &SessionOverlay::toggle);
}
