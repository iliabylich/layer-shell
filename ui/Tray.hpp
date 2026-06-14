#pragma once

#include "Event.hpp"
#include <QHash>
#include <QObject>

class TrayButtonWithMenu;
class UiModel;
class QBoxLayout;

class TrayRegistry : public QObject {
  Q_OBJECT

public:
  explicit TrayRegistry(UiModel *model, QWidget *parent, QBoxLayout *layout);

public Q_SLOTS:
  void add(const QString &app_id, const QIcon &icon,
           const QVector<Event::Tray::MenuItem> &items);
  void updateIcon(const QString &app_id, const QIcon &icon);
  void updateMenu(const QString &app_id,
                  const QVector<Event::Tray::MenuItem> &items);
  void remove(const QString &app_id);

private:
  QHash<QString, TrayButtonWithMenu *> data;
  UiModel *model;
  QWidget *parent;
  QBoxLayout *layout;
};
