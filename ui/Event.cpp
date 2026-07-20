#include "Event.hpp"
#include "bindings.hpp"
#include <QCoreApplication>
#include <QDebug>

Event::Memory::Memory(const IoEvent::Memory_Body &data)
    : used(data.used), total(data.total) {}

Event::CPU::CPU(const IoEvent::CpuUsage_Body &data) {
  for (size_t i = 0; i < data.usage_per_core.count; i++) {
    usage_per_core.push_front(data.usage_per_core.items[i]);
  }
}

Event::Time::Time(const IoEvent::Time_Body &data) : now(data.now) {}

Event::Language::Language(const IoEvent::Language_Body &data)
    : lang(data.lang) {}

Event::Weather::Weather(const IoEvent::Weather_Body &data)
    : temperature(data.temperature), code(data.code) {
  for (size_t i = 0; i < Event::Weather::OnHour::COUNT; i++) {
    struct WeatherOnHour w = data.hourly_forecast[i];

    hourly_forecast[i] = Event::Weather::OnHour{
        .unix_seconds = w.unix_seconds,
        .temperature = w.temperature,
        .code = w.code,
    };
  }

  for (size_t i = 0; i < Event::Weather::OnDay::COUNT; i++) {
    struct WeatherOnDay w = data.daily_forecast[i];

    daily_forecast[i] = Event::Weather::OnDay{
        .unix_seconds = w.unix_seconds,
        .temperature_min = w.temperature_min,
        .temperature_max = w.temperature_max,
        .code = w.code,
    };
  }
}

Event::Network::Network(const IoEvent::NetworkSsidAndStrength_Body &data)
    : ssid(data.ssid), strength(data.strength) {}

Event::UploadSpeed::UploadSpeed(const IoEvent::UploadSpeed_Body &data)
    : bytes_per_sec(data.bytes_per_sec) {}

Event::DownloadSpeed::DownloadSpeed(const IoEvent::DownloadSpeed_Body &data)
    : bytes_per_sec(data.bytes_per_sec) {}

const Event::Tray::MenuItem::Value &Event::Tray::MenuItem::value() const {
  return value_;
}

QString qstring_from_label(TrayLabel label) {
  return QString::fromUtf8((const char *)label.buf, label.len);
}

Event::Tray::MenuItem map_item_at(const TrayMenu &menu, size_t idx,
                                  uint32_t service);

QVector<Event::Tray::MenuItem> map_children_at(const TrayMenu &menu,
                                               size_t start_idx, size_t end_idx,
                                               uint32_t service) {
  QVector<Event::Tray::MenuItem> out;
  for (size_t idx = start_idx; idx < end_idx; idx++) {
    out.push_back(map_item_at(menu, idx, service));
  }
  return out;
}

Event::Tray::MenuItem map_item_at(const TrayMenu &menu, size_t idx,
                                  uint32_t service) {
  auto item = menu._0[idx].element;
  switch (item.tag) {
  case TrayElement::Tag::Regular:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Regular{
            .id = item.regular.id,
            .service = service,
            .label = qstring_from_label(item.regular.label),
        })};
  case TrayElement::Tag::Disabled:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Disabled{
            .id = item.disabled.id,
            .service = service,
            .label = qstring_from_label(item.disabled.label),
        })};
  case TrayElement::Tag::Checkbox:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Checkbox{
            .id = item.checkbox.id,
            .service = service,
            .label = qstring_from_label(item.checkbox.label),
            .checked = item.checkbox.checked,
        })};
  case TrayElement::Tag::Radio:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Radio{
            .id = item.radio.id,
            .service = service,
            .label = qstring_from_label(item.radio.label),
            .selected = item.radio.selected,
        })};
  case TrayElement::Tag::Nested:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Nested{
            .id = item.nested.id,
            .service = service,
            .label = qstring_from_label(item.nested.label),
            .children = map_children_at(menu, item.nested.child_start_idx,
                                        item.nested.child_end_idx, service),
        })};
  case TrayElement::Tag::Section:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Section{
            .children = map_children_at(menu, item.section.child_start_idx,
                                        item.section.child_end_idx, service),
        })};
  case TrayElement::Tag::None:
    break;
  }

  qDebug() << "unknown TrayItem::Tag " << static_cast<int>(item.tag) << "\n";
  std::abort();
}

QVector<Event::Tray::MenuItem> map_root_items(const TrayMenu &menu,
                                              uint32_t service) {
  QVector<Event::Tray::MenuItem> out;
  for (size_t i = 0; i < TRAY_MENU_ITEMS_COUNT; i++) {
    auto item = menu._0[i];
    if (!item.root) {
      break;
    }
    out.push_back(map_item_at(menu, i, service));
  }
  return out;
}

Event::Tray::AppAdded::AppAdded(const IoEvent::TrayAppAdded_Body &data)
    : service(data.service), items(map_root_items(data.menu, data.service)),
      icon(data.icon) {}
Event::Tray::AppIconUpdated::AppIconUpdated(
    const IoEvent::TrayAppIconUpdated_Body &data)
    : service(data.service), icon(data.icon) {}
Event::Tray::AppMenuUpdated::AppMenuUpdated(
    const IoEvent::TrayAppMenuUpdated_Body &data)
    : service(data.service), items(map_root_items(data.menu, data.service)) {}
Event::Tray::AppRemoved::AppRemoved(const IoEvent::TrayAppRemoved_Body &data)
    : service(data.service) {}
Event::KbModToggled::KbModToggled(const IoEvent::KbModToggled_Body &data)
    : enabled(data.enabled) {
  switch (data.kind) {
  case KbModKind::CapsLock:
    kind = Event::KbModToggled::Kind::CapsLock;
    break;
  case KbModKind::NumLock:
    kind = Event::KbModToggled::Kind::NumLock;
    break;
  default:
    qDebug() << "unknown KbModKind " << static_cast<int>(data.kind) << "\n";
    std::abort();
  }
}
Event::Sound::Sound(const IoEvent::Sound_Body &data)
    : volume(data.volume), muted(data.muted) {}

Event Event::from(const IoEvent &event) {
  switch (event.tag) {
  case IoEvent::Tag::Memory:
    return Event(event.memory);
  case IoEvent::Tag::CpuUsage:
    return Event(event.cpu_usage);
  case IoEvent::Tag::Time:
    return Event(event.time);
  case IoEvent::Tag::Language:
    return Event(event.language);
  case IoEvent::Tag::Weather:
    return Event(event.weather);
  case IoEvent::Tag::NetworkSsidAndStrength:
    return Event(event.network_ssid_and_strength);
  case IoEvent::Tag::UploadSpeed:
    return Event(event.upload_speed);
  case IoEvent::Tag::DownloadSpeed:
    return Event(event.download_speed);
  case IoEvent::Tag::TrayAppAdded:
    return Event(event.tray_app_added);
  case IoEvent::Tag::TrayAppIconUpdated:
    return Event(event.tray_app_icon_updated);
  case IoEvent::Tag::TrayAppMenuUpdated:
    return Event(event.tray_app_menu_updated);
  case IoEvent::Tag::TrayAppRemoved:
    return Event(event.tray_app_removed);
  case IoEvent::Tag::ToggleSessionScreen:
    return Event(Event::ToggleSessionScreen());
  case IoEvent::Tag::KbModToggled:
    return Event(event.kb_mod_toggled);
  case IoEvent::Tag::Exit:
    return Event(Event::Exit());
  case IoEvent::Tag::Sound:
    return Event(event.sound);
  }

  qDebug() << "unknown IoEvent::Tag " << static_cast<int>(event.tag) << "\n";
  std::abort();
}

Event::Event(Value value) : value_(value) {}

const Event::Value &Event::value() const { return value_; }
