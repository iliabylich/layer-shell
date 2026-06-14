#pragma once

#include <QWidget>

class UiModel;
class QBoxLayout;
namespace LayerShellQt {
class Window;
}
class QSize;

class Overlay : public QWidget {
  Q_OBJECT

public:
  explicit Overlay(UiModel *model);

public Q_SLOTS:
  void toggle();

protected:
  void keyPressEvent(QKeyEvent *event) override;
  UiModel *model;
  LayerShellQt::Window *layer = nullptr;
  void setCloseOnEscape(bool close_on_escape);
  QBoxLayout *initCenteredLayout(const QSize &size, const QString &name);

private:
  bool close_on_escape_ = false;
};
