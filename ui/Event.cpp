#include "Event.hpp"
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

Event::Tray::MenuItem::Regular::Regular(
    const IO_TrayItem::IO_Regular_Body &data)
    : id(data.id), uuid(data.uuid), label(data.label) {}
Event::Tray::MenuItem::Disabled::Disabled(
    const IO_TrayItem::IO_Disabled_Body &data)
    : id(data.id), uuid(data.uuid), label(data.label) {}
Event::Tray::MenuItem::Checkbox::Checkbox(
    const IO_TrayItem::IO_Checkbox_Body &data)
    : id(data.id), uuid(data.uuid), label(data.label), checked(data.checked) {}
Event::Tray::MenuItem::Radio::Radio(const IO_TrayItem::IO_Radio_Body &data)
    : id(data.id), uuid(data.uuid), label(data.label), selected(data.selected) {

}
Event::Tray::MenuItem::Nested::Nested(const IO_TrayItem::IO_Nested_Body &data)
    : id(data.id), uuid(data.uuid), label(data.label),
      children(Event::Tray::MenuItem::Many(data.children)) {}
Event::Tray::MenuItem::Section::Section(
    const IO_TrayItem::IO_Section_Body &data)
    : children(Event::Tray::MenuItem::Many(data.children)) {}

Event::Tray::MenuItem Event::Tray::MenuItem::from(const IO_TrayItem &item) {
  switch (item.tag) {
  case IO_TrayItem::Tag::Regular:
    return Event::Tray::MenuItem(item.regular);
  case IO_TrayItem::Tag::Disabled:
    return Event::Tray::MenuItem(item.disabled);
  case IO_TrayItem::Tag::Checkbox:
    return Event::Tray::MenuItem(item.checkbox);
  case IO_TrayItem::Tag::Radio:
    return Event::Tray::MenuItem(item.radio);
  case IO_TrayItem::Tag::Nested:
    return Event::Tray::MenuItem(item.nested);
  case IO_TrayItem::Tag::Section:
    return Event::Tray::MenuItem(item.section);
  }

  qDebug() << "unknown IO_TrayItem::Tag " << static_cast<int>(item.tag) << "\n";
  std::abort();
}

Event::Tray::MenuItem::MenuItem(Value value) : value_(value) {}

QVector<Event::Tray::MenuItem>
Event::Tray::MenuItem::Many(const IO_FFIArray<IO_TrayItem> &items) {
  QVector<Event::Tray::MenuItem> out;
  for (size_t i = 0; i < items.len; i++) {
    out.push_back(Event::Tray::MenuItem::from(items.ptr[i]));
  }
  return out;
}

QIcon qicon_from_io_tray_icon(const IO_TrayIcon &icon) {
  switch (icon.tag) {
  case IO_TrayIcon::Tag::Path:
    return QIcon(icon.path.path);
  case IO_TrayIcon::Tag::Name:
    return QIcon::fromTheme(icon.name.name);
  case IO_TrayIcon::Tag::Pixmap: {
    auto pixmap = icon.pixmap._0;
    auto width = pixmap.width;
    auto height = pixmap.height;
    QImage image(pixmap.bytes.ptr, width, height, width * 4,
                 QImage::Format_RGBA8888);
    return QIcon(QPixmap::fromImage(image.copy()));
  }
  case IO_TrayIcon::Tag::Unset:
    return QIcon::fromTheme("process-stop");
  }

  qDebug() << "unknown IO_TrayIcon::Tag " << static_cast<int>(icon.tag) << "\n";
  std::abort();
}

Event::Tray::AppAdded::AppAdded(const IO_Event::IO_TrayAppAdded_Body &data)
    : service(data.service), items(Event::Tray::MenuItem::Many(data.items)),
      icon(qicon_from_io_tray_icon(data.icon)) {}
Event::Tray::AppIconUpdated::AppIconUpdated(
    const IO_Event::IO_TrayAppIconUpdated_Body &data)
    : service(data.service), icon(qicon_from_io_tray_icon(data.icon)) {}
Event::Tray::AppMenuUpdated::AppMenuUpdated(
    const IO_Event::IO_TrayAppMenuUpdated_Body &data)
    : service(data.service), items(Event::Tray::MenuItem::Many(data.items)) {}
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
