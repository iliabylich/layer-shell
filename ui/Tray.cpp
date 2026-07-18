#include "Tray.hpp"
#include "UiModel.hpp"
#include <QFrame>
#include <QHBoxLayout>
#include <QMenu>
#include <QToolButton>

class Visitor {
public:
  static void visit(QMenu *menu, UiModel *model,
                    const QVector<Event::Tray::MenuItem> &items) {
    for (const auto &item : items) {
      std::visit(
          Visitor{
              menu,
              model,
          },
          item.value());
    }
  }

  void operator()(const Event::Tray::MenuItem::Regular &item) {
    QAction *action = menu->addAction(item.label);
    QObject::connect(action, &QAction::triggered, model,
                     [model = this->model, service = item.service,
                      id = item.id] { model->triggerTrayItem(service, id); });
  }

  void operator()(const Event::Tray::MenuItem::Disabled &item) const {
    QAction *action = menu->addAction(item.label);
    action->setEnabled(false);
  }
  void operator()(const Event::Tray::MenuItem::Checkbox &item) const {
    QAction *action = menu->addAction(item.label);
    action->setCheckable(true);
    action->setChecked(item.checked);
    QObject::connect(action, &QAction::triggered, model,
                     [model = this->model, service = item.service,
                      id = item.id] { model->triggerTrayItem(service, id); });
  }
  void operator()(const Event::Tray::MenuItem::Radio &item) const {
    QAction *action = menu->addAction(item.label);
    action->setCheckable(true);
    action->setChecked(item.selected);
    QObject::connect(action, &QAction::triggered, model,
                     [model = this->model, service = item.service,
                      id = item.id] { model->triggerTrayItem(service, id); });
  }
  void operator()(const Event::Tray::MenuItem::Nested &item) const {
    auto *submenu = new QMenu(item.label, menu);
    Visitor::visit(submenu, model, item.children);
    menu->addAction(submenu->menuAction());
  }
  void operator()(const Event::Tray::MenuItem::Section &item) const {
    if (!menu->isEmpty()) {
      menu->addSeparator();
    }
    Visitor::visit(menu, model, item.children);
    if (!item.children.isEmpty()) {
      menu->addSeparator();
    }
  }

  QMenu *menu;
  UiModel *model;
};

class TrayButtonWithMenu : public QToolButton {
public:
  TrayButtonWithMenu(QWidget *parent, UiModel *model, const QIcon &icon,
                     const QVector<Event::Tray::MenuItem> &items)
      : QToolButton(parent), model(model) {
    setCursor(Qt::PointingHandCursor);
    setIconSize(QSize(24, 24));
    setFixedSize(32, 32);
    setPopupMode(QToolButton::InstantPopup);
    menu = new QMenu(this);
    Visitor::visit(menu, model, items);
    setMenu(menu);
    setIcon(icon);
  }
  QMenu *getMenu() const { return menu; }
  void updateMenu(const QVector<Event::Tray::MenuItem> &items) {
    menu->clear();
    Visitor::visit(menu, model, items);
  }

private:
  UiModel *model;
  QMenu *menu;
};

TrayRegistry::TrayRegistry(UiModel *model, QWidget *parent, QBoxLayout *layout)
    : QObject(parent), model(model), parent(parent), layout(layout) {};

void TrayRegistry::add(uint32_t service, const QIcon &icon,
                       const QVector<Event::Tray::MenuItem> &items) {
  auto item = new TrayButtonWithMenu(parent, model, icon, items);
  layout->addWidget(item);
  data.insert(service, item);
}

void TrayRegistry::updateIcon(uint32_t service, const QIcon &icon) {
  auto it = data.find(service);
  if (it == data.end()) {
    return;
  }
  it.value()->setIcon(icon);
}

void TrayRegistry::updateMenu(uint32_t service,
                              const QVector<Event::Tray::MenuItem> &items) {
  auto it = data.find(service);
  if (it == data.end()) {
    return;
  }

  it.value()->updateMenu(items);
}

void TrayRegistry::remove(uint32_t service) {
  auto it = data.find(service);
  if (it == data.end()) {
    return;
  }
  delete it.value();
  data.erase(it);
}
