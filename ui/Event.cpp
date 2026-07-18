#include "Event.hpp"
#include "bindings.hpp"
#include <QCoreApplication>
#include <QDebug>

Event::Memory::Memory(const IO_Event::IO_Memory_Body &data)
    : used(data.used), total(data.total) {}

Event::CPU::CPU(const IO_Event::IO_CpuUsage_Body &data) {
  for (size_t i = 0; i < data.usage_per_core.len; i++) {
    usage_per_core.push_front(data.usage_per_core.ptr[i]);
  }
}

Event::Time::Time(const IO_Event::IO_Time_Body &data) : now(data.now) {}

Event::Language::Language(const IO_Event::IO_Language_Body &data)
    : lang(data.lang) {}

Event::Weather::Weather(const IO_Event::IO_Weather_Body &data)
    : temperature(data.temperature), code(data.code) {
  for (size_t i = 0; i < Event::Weather::OnHour::COUNT; i++) {
    struct IO_WeatherOnHour w = data.hourly_forecast[i];

    hourly_forecast[i] = Event::Weather::OnHour{
        .unix_seconds = w.unix_seconds,
        .temperature = w.temperature,
        .code = w.code,
    };
  }

  for (size_t i = 0; i < Event::Weather::OnDay::COUNT; i++) {
    struct IO_WeatherOnDay w = data.daily_forecast[i];

    daily_forecast[i] = Event::Weather::OnDay{
        .unix_seconds = w.unix_seconds,
        .temperature_min = w.temperature_min,
        .temperature_max = w.temperature_max,
        .code = w.code,
    };
  }
}

Event::Network::Network(const IO_Event::IO_NetworkSsidAndStrength_Body &data)
    : ssid(data.ssid), strength(data.strength) {}

Event::UploadSpeed::UploadSpeed(const IO_Event::IO_UploadSpeed_Body &data)
    : bytes_per_sec(data.bytes_per_sec) {}

Event::DownloadSpeed::DownloadSpeed(const IO_Event::IO_DownloadSpeed_Body &data)
    : bytes_per_sec(data.bytes_per_sec) {}

const Event::Tray::MenuItem::Value &Event::Tray::MenuItem::value() const {
  return value_;
}

QString qstring_from_label(IO_TrayLabel label) {
  return QString::fromUtf8((const char *)label.buf, label.len);
}

Event::Tray::MenuItem map_item_at(const IO_TrayMenu &menu, size_t idx,
                                  uint32_t service);

QVector<Event::Tray::MenuItem> map_children_at(const IO_TrayMenu &menu,
                                               size_t start_idx, size_t end_idx,
                                               uint32_t service) {
  QVector<Event::Tray::MenuItem> out;
  for (size_t idx = start_idx; idx < end_idx; idx++) {
    out.push_back(map_item_at(menu, idx, service));
  }
  return out;
}

Event::Tray::MenuItem map_item_at(const IO_TrayMenu &menu, size_t idx,
                                  uint32_t service) {
  auto item = menu._0[idx].element;
  switch (item.tag) {
  case IO_TrayElement::Tag::Regular:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Regular{
            .id = item.regular.id,
            .service = service,
            .label = qstring_from_label(item.regular.label),
        })};
  case IO_TrayElement::Tag::Disabled:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Disabled{
            .id = item.disabled.id,
            .service = service,
            .label = qstring_from_label(item.disabled.label),
        })};
  case IO_TrayElement::Tag::Checkbox:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Checkbox{
            .id = item.checkbox.id,
            .service = service,
            .label = qstring_from_label(item.checkbox.label),
            .checked = item.checkbox.checked,
        })};
  case IO_TrayElement::Tag::Radio:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Radio{
            .id = item.radio.id,
            .service = service,
            .label = qstring_from_label(item.radio.label),
            .selected = item.radio.selected,
        })};
  case IO_TrayElement::Tag::Nested:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Nested{
            .id = item.nested.id,
            .service = service,
            .label = qstring_from_label(item.nested.label),
            .children = map_children_at(menu, item.nested.child_start_idx,
                                        item.nested.child_end_idx, service),
        })};
  case IO_TrayElement::Tag::Section:
    return Event::Tray::MenuItem{
        .value_ = Event::Tray::MenuItem::Value(Event::Tray::MenuItem::Section{
            .children = map_children_at(menu, item.section.child_start_idx,
                                        item.section.child_end_idx, service),
        })};
  case IO_TrayElement::Tag::None:
    break;
  }

  qDebug() << "unknown IO_TrayItem::Tag " << static_cast<int>(item.tag) << "\n";
  std::abort();
}

QVector<Event::Tray::MenuItem> map_root_items(const IO_TrayMenu &menu,
                                              uint32_t service) {
  QVector<Event::Tray::MenuItem> out;
  for (size_t i = 0; i < IO_TRAY_MENU_ITEMS_COUNT; i++) {
    auto item = menu._0[i];
    if (!item.root) {
      break;
    }
    out.push_back(map_item_at(menu, i, service));
  }
  return out;
}

Event::Tray::AppAdded::AppAdded(const IO_Event::IO_TrayAppAdded_Body &data)
    : service(data.service), items(map_root_items(data.menu, data.service)),
      icon(data.icon) {}
Event::Tray::AppIconUpdated::AppIconUpdated(
    const IO_Event::IO_TrayAppIconUpdated_Body &data)
    : service(data.service), icon(data.icon) {}
Event::Tray::AppMenuUpdated::AppMenuUpdated(
    const IO_Event::IO_TrayAppMenuUpdated_Body &data)
    : service(data.service), items(map_root_items(data.menu, data.service)) {}
Event::Tray::AppRemoved::AppRemoved(
    const IO_Event::IO_TrayAppRemoved_Body &data)
    : service(data.service) {}
Event::KbModToggled::KbModToggled(const IO_Event::IO_KbModToggled_Body &data)
    : enabled(data.enabled) {
  switch (data.kind) {
  case IO_KbModKind::CapsLock:
    kind = Event::KbModToggled::Kind::CapsLock;
    break;
  case IO_KbModKind::NumLock:
    kind = Event::KbModToggled::Kind::NumLock;
    break;
  default:
    qDebug() << "unknown IO_KbModKind " << static_cast<int>(data.kind) << "\n";
    std::abort();
  }
}
Event::Sound::Sound(const IO_Event::IO_Sound_Body &data)
    : volume(data.volume), muted(data.muted) {}

Event Event::from(const IO_Event &event) {
  switch (event.tag) {
  case IO_Event::Tag::Memory:
    return Event(event.memory);
  case IO_Event::Tag::CpuUsage:
    return Event(event.cpu_usage);
  case IO_Event::Tag::Time:
    return Event(event.time);
  case IO_Event::Tag::Language:
    return Event(event.language);
  case IO_Event::Tag::Weather:
    return Event(event.weather);
  case IO_Event::Tag::NetworkSsidAndStrength:
    return Event(event.network_ssid_and_strength);
  case IO_Event::Tag::UploadSpeed:
    return Event(event.upload_speed);
  case IO_Event::Tag::DownloadSpeed:
    return Event(event.download_speed);
  case IO_Event::Tag::TrayAppAdded:
    return Event(event.tray_app_added);
  case IO_Event::Tag::TrayAppIconUpdated:
    return Event(event.tray_app_icon_updated);
  case IO_Event::Tag::TrayAppMenuUpdated:
    return Event(event.tray_app_menu_updated);
  case IO_Event::Tag::TrayAppRemoved:
    return Event(event.tray_app_removed);
  case IO_Event::Tag::ToggleSessionScreen:
    return Event(Event::ToggleSessionScreen());
  case IO_Event::Tag::KbModToggled:
    return Event(event.kb_mod_toggled);
  case IO_Event::Tag::Exit:
    return Event(Event::Exit());
  case IO_Event::Tag::Sound:
    return Event(event.sound);
  }

  qDebug() << "unknown IO_Event::Tag " << static_cast<int>(event.tag) << "\n";
  std::abort();
}

Event::Event(Value value) : value_(value) {}

const Event::Value &Event::value() const { return value_; }
