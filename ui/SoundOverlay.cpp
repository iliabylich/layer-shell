#include "SoundOverlay.hpp"
#include "Overlay.hpp"
#include "UiModel.hpp"

#include <LayerShellQt/Window>
#include <QColor>
#include <QHBoxLayout>
#include <QLabel>
#include <QPaintEvent>
#include <QPainter>
#include <QPainterPath>
#include <QTimer>
#include <QWindow>

class VolumeBar : public QWidget {
public:
  explicit VolumeBar(QWidget *parent = nullptr) : QWidget(parent) {
    setFixedSize(300, 64);
  }

  void setValue(int value) {
    value_ = std::clamp(value, 0, 100);
    update();
  }

protected:
  void paintEvent(QPaintEvent *) override {
    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing);

    QRectF rect(0, 0, width(), height());
    QPainterPath clip;
    clip.addRoundedRect(rect, 12, 12);

    painter.fillPath(clip, QColor(19, 20, 18, 153));

    QRectF filled = rect;
    filled.setWidth(rect.width() * value_ / 100.0);
    painter.save();
    painter.setClipPath(clip);
    painter.fillRect(filled, QColor("#5a614f"));
    painter.restore();
  }

private:
  int value_ = 0;
};

//

SoundOverlay::SoundOverlay(UiModel *model) : Overlay(model) {
  auto *layout = initCenteredLayout(QSize(380, 100), "soundOverlay");
  layout->setContentsMargins(15, 15, 15, 15);
  layout->setSpacing(20);

  icon_ = new QLabel;
  icon_->setObjectName("soundIcon");
  icon_->setAlignment(Qt::AlignCenter);
  layout->addWidget(icon_);

  bar_ = new VolumeBar;
  layout->addWidget(bar_);

  hide_timer_ = new QTimer(this);
  hide_timer_->setSingleShot(true);
  hide_timer_->setInterval(1000);
  connect(model, &UiModel::soundChanged, this,
          [this](uint8_t volume, const QString &icon) {
            bar_->setValue(volume);
            icon_->setText(icon);
            show();
            hide_timer_->start();
          });
  connect(hide_timer_, &QTimer::timeout, this, &QWidget::hide);

  layer->setScope("LayerShell/Sound");
  layer->setLayer(LayerShellQt::Window::LayerOverlay);
  layer->setActivateOnShow(false);
  layer->setKeyboardInteractivity(
      LayerShellQt::Window::KeyboardInteractivityNone);
  layer->setAnchors(
      LayerShellQt::Window::Anchors(LayerShellQt::Window::AnchorBottom));
  layer->setMargins(QMargins(0, 0, 0, 100));
}
