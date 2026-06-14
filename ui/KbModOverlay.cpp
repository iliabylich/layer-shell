#include "KbModOverlay.hpp"
#include "UiModel.hpp"

#include <LayerShellQt/Window>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QTimer>
#include <QWindow>

KbModOverlay::KbModOverlay(UiModel *model) : Overlay(model) {
  auto *surface_layout = new QHBoxLayout(this);
  surface_layout->setContentsMargins(0, 0, 0, 0);

  auto *frame = new QFrame;
  frame->setObjectName("kbModOverlay");
  surface_layout->addWidget(frame);

  auto *layout = new QHBoxLayout(frame);
  layout->setContentsMargins(15, 15, 15, 15);
  layout->setSpacing(20);

  icon = new QLabel;
  icon->setObjectName("kbModIcon");
  icon->setAlignment(Qt::AlignCenter);
  layout->addWidget(icon);

  status = new QLabel;
  status->setObjectName("kbModStatus");
  status->setAlignment(Qt::AlignCenter);
  layout->addWidget(status);

  timer = new QTimer(this);
  timer->setSingleShot(true);
  timer->setInterval(1000);
  connect(timer, &QTimer::timeout, this, &QWidget::hide);
  connect(model, &UiModel::kbModChanged, this,
          [this](const QString &icon, const QString &text) {
            showTemporarily(icon, text);
          });

  layer->setScope("LayerShell/KbMod");
  layer->setLayer(LayerShellQt::Window::LayerOverlay);
  layer->setActivateOnShow(false);
  layer->setKeyboardInteractivity(
      LayerShellQt::Window::KeyboardInteractivityNone);
  layer->setAnchors(
      LayerShellQt::Window::Anchors(LayerShellQt::Window::AnchorBottom));
  layer->setMargins(QMargins(0, 0, 0, 100));
}

void KbModOverlay::showTemporarily(const QString &iconText,
                                   const QString &statusText) {
  icon->setText(iconText);
  status->setText(statusText);
  show();
  raise();
  timer->start();
}
