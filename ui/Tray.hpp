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
  void add(uint32_t service, const QIcon &icon,
           const QVector<Event::Tray::MenuItem> &items);
  void updateIcon(uint32_t service, const QIcon &icon);
  void updateMenu(uint32_t service,
                  const QVector<Event::Tray::MenuItem> &items);
  void remove(uint32_t service);

private:
  QHash<uint32_t, TrayButtonWithMenu *> data;
  UiModel *model;
  QWidget *parent;
  QBoxLayout *layout;
};
