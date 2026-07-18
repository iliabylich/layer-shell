#include "UiModel.hpp"
#include "Event.hpp"
#include "bindings.hpp"
#include <QSocketNotifier>

namespace fmt {

QString network_speed(uint64_t bytes_per_sec) {
  enum class Unit {
    B,
    KB,
    MB,
  };

  uint64_t value = bytes_per_sec;
  Unit unit = Unit::B;

  if (value / 1024 > 0) {
    value /= 1024;
    unit = Unit::KB;
    if (value / 1024 > 0) {
      value /= 1024;
      unit = Unit::MB;
    }
  }

  const char *suffix = unit == Unit::B    ? "B/s"
                       : unit == Unit::KB ? "KB/s"
                                          : "MB/s";
  return QStringLiteral("%1%2").arg(value).arg(QString::fromUtf8(suffix));
}

QString weather_description(IO_WeatherCode code) {
  switch (code) {
  case IO_WeatherCode::ClearSky:
    return "Clear Sky";
  case IO_WeatherCode::MainlyClear:
    return "Mainly Clear";
  case IO_WeatherCode::PartlyCloudy:
    return "Partly Cloudy";
  case IO_WeatherCode::Overcast:
    return "Overcast";
  case IO_WeatherCode::FogDepositingRime:
    return "Fog (Depositing Rime)";
  case IO_WeatherCode::FogNormal:
    return "Fog (Normal)";
  case IO_WeatherCode::DrizzleLight:
    return "Drizzle (Light)";
  case IO_WeatherCode::DrizzleModerate:
    return "Drizzle (Moderate)";
  case IO_WeatherCode::DrizzleDense:
    return "Drizzle (Dense)";
  case IO_WeatherCode::FreezingDrizzleLight:
    return "Freezing Drizzle (Light)";
  case IO_WeatherCode::FreezingDrizzleDense:
    return "Freezing Drizzle (Dense)";
  case IO_WeatherCode::RainSlight:
    return "Rain (Slight)";
  case IO_WeatherCode::RainModerate:
    return "Rain (Moderate)";
  case IO_WeatherCode::RainHeavy:
    return "Rain (Heavy)";
  case IO_WeatherCode::FreezingRainLight:
    return "Freezing Rain (Light)";
  case IO_WeatherCode::FreezingRainHeavy:
    return "Freezing Rain (Heavy)";
  case IO_WeatherCode::SnowFallSlight:
    return "Snow Fall (Slight)";
  case IO_WeatherCode::SnowFallModerate:
    return "Snow Fall (Moderate)";
  case IO_WeatherCode::SnowFallHeavy:
    return "Snow Fall (Heavy)";
  case IO_WeatherCode::SnowGrains:
    return "Snow Grains";
  case IO_WeatherCode::RainShowersSlight:
    return "Rain Showers (Slight)";
  case IO_WeatherCode::RainShowersModerate:
    return "Rain Showers (Moderate)";
  case IO_WeatherCode::RainShowersViolent:
    return "Rain Showers (Violent)";
  case IO_WeatherCode::SnowShowersSlight:
    return "Snow Showers (Slight)";
  case IO_WeatherCode::SnowShowersHeavy:
    return "Snow Showers (Heavy)";
  case IO_WeatherCode::Thunderstorm:
    return "Thunderstorm";
  case IO_WeatherCode::ThunderstormWithHailSight:
    return "Thunderstorm With Hail (Sight)";
  case IO_WeatherCode::ThunderstormWithHailHeavy:
    return "Thunderstorm With Hail (Heavy)";
  case IO_WeatherCode::Unknown:
  default:
    return "Unknown";
  }
}

QString weather_icon(IO_WeatherCode code) {
  switch (code) {
  case IO_WeatherCode::ClearSky:
  case IO_WeatherCode::MainlyClear:
    return "󰖙";
  case IO_WeatherCode::PartlyCloudy:
  case IO_WeatherCode::Overcast:
    return "󰖐";
  case IO_WeatherCode::FogDepositingRime:
  case IO_WeatherCode::FogNormal:
    return "󰖑";
  case IO_WeatherCode::DrizzleDense:
  case IO_WeatherCode::DrizzleLight:
  case IO_WeatherCode::DrizzleModerate:
  case IO_WeatherCode::FreezingDrizzleLight:
  case IO_WeatherCode::FreezingDrizzleDense:
  case IO_WeatherCode::RainSlight:
  case IO_WeatherCode::RainModerate:
  case IO_WeatherCode::RainHeavy:
  case IO_WeatherCode::FreezingRainLight:
  case IO_WeatherCode::FreezingRainHeavy:
  case IO_WeatherCode::RainShowersSlight:
  case IO_WeatherCode::RainShowersModerate:
  case IO_WeatherCode::RainShowersViolent:
    return "󰖗";
  case IO_WeatherCode::SnowFallSlight:
  case IO_WeatherCode::SnowFallModerate:
  case IO_WeatherCode::SnowFallHeavy:
  case IO_WeatherCode::SnowGrains:
  case IO_WeatherCode::SnowShowersSlight:
  case IO_WeatherCode::SnowShowersHeavy:
    return "󰖘";
  case IO_WeatherCode::Thunderstorm:
  case IO_WeatherCode::ThunderstormWithHailSight:
  case IO_WeatherCode::ThunderstormWithHailHeavy:
    return "";
  case IO_WeatherCode::Unknown:
  default:
    return "";
  }
}

QString kb_mod_label(Event::KbModToggled::Kind kind, bool enabled) {
  QString name =
      kind == Event::KbModToggled::Kind::NumLock ? "NumLock" : "CapsLock";
  return QStringLiteral("%1 %2").arg(name, enabled ? "ON" : "OFF");
}

QString cpu(const QList<uint8_t> &usage_per_core) {
  constexpr const char *CpuBlocks[] = {
      "&#9601;", "&#9602;", "&#9603;", "&#9604;",
      "&#9605;", "&#9606;", "&#9607;", "&#9608;",
  };

  constexpr const char *CpuColors[] = {
      "#FFFFFF", "#FFD5D5", "#FFAAAA", "#FF8080",
      "#FF5555", "#FF2B2B", "#FF0000", "#E60000",
  };

  constexpr size_t CpuIndicatorCount = sizeof(CpuBlocks) / sizeof(const char *);

  QString text;
  for (qsizetype i = 0; i < usage_per_core.size(); i++) {
    size_t idx =
        static_cast<size_t>(usage_per_core[i] / 100.0 * CpuIndicatorCount);
    if (idx >= CpuIndicatorCount) {
      idx = CpuIndicatorCount - 1;
    }

    if (!text.isEmpty()) {
      text += "&thinsp;";
    }
    text += QStringLiteral("<span style=\"color:%1\">%2</span>")
                .arg(QString::fromUtf8(CpuColors[idx]),
                     QString::fromUtf8(CpuBlocks[idx]));
  }
  return text;
}

QString memory(double used, double total) {
  return QStringLiteral("RAM %1G/%2G")
      .arg(used, 0, 'f', 1)
      .arg(total, 0, 'f', 1);
}

QString network_name(const QString &ssid, uint8_t strength) {
  return QStringLiteral("%1 (%2)%").arg(ssid).arg(strength);
}

QString weather_summary(float temperature, IO_WeatherCode code) {
  return QStringLiteral("%1℃ %2")
      .arg(static_cast<double>(temperature), 0, 'f', 1)
      .arg(weather_description(code));
}

QString sound(uint8_t volume, bool muted) {
  QString icon = "";
  if (volume == 0 || muted) {
    icon = "󰝟";
  } else if (volume <= 33) {
    icon = "󰕿";
  } else if (volume <= 66) {
    icon = "󰖀";
  } else {
    icon = "󰕾";
  }
  return icon;
}

} // namespace fmt

UiModel::UiModel(QObject *parent) : QObject(parent) {
  io = io_init(eventReceived, this);

  io_notifier_ =
      new QSocketNotifier(io_as_raw_fd(io), QSocketNotifier::Read, this);
  connect(io_notifier_, &QSocketNotifier::activated, this,
          [this] { io_handle_readable(io); });
}

IO_IO *UiModel::getIO() const { return io; }

UiModel::~UiModel() { io_deinit(io); }

void UiModel::changeWallpaper() { io_change_wallpaper(io); }

void UiModel::lock() { io_lock(io); }

void UiModel::logout() { io_logout(io); }

void UiModel::reboot() { io_reboot(io); }

void UiModel::shutdown() { io_shutdown(io); }

void UiModel::spawnBluetoothEditor() { io_spawn_bluetooh_editor(io); }

void UiModel::spawnWifiEditor() { io_spawn_wifi_editor(io); }

void UiModel::spawnSystemMonitor() { io_spawn_system_monitor(io); }

void UiModel::triggerTrayItem(uint32_t service, uint32_t id) {
  io_trigger_tray(io, service, id);
}

void UiModel::eventReceived(const IO_Event *event, void *data) {
  auto self = reinterpret_cast<UiModel *>(data);
  self->handleEvent(*event);
}

void UiModel::handleEvent(const IO_Event &event) {
  std::visit([this](const auto &event) { (*this)(event); },
             Event::from(event).value());
}

void UiModel::operator()(const Event::Memory &e) {
  Q_EMIT memoryTextChanged(fmt::memory(e.used, e.total));
}
void UiModel::operator()(const Event::CPU &e) {
  Q_EMIT cpuTextChanged(fmt::cpu(e.usage_per_core));
}
void UiModel::operator()(const Event::Time &e) {
  Q_EMIT timeTextChanged(e.now);
}
void UiModel::operator()(const Event::Language &e) {
  Q_EMIT languageTextChanged(e.lang);
}
void UiModel::operator()(const Event::Weather &e) {
  QString summary = fmt::weather_summary(e.temperature, e.code);

  std::array<WeatherHourForecast, Event::Weather::OnHour::COUNT> hourly;
  for (size_t i = 0; i < Event::Weather::OnHour::COUNT; i++) {
    hourly[i] = {
        .unix_seconds = e.hourly_forecast[i].unix_seconds,
        .temperature = e.hourly_forecast[i].temperature,
        .icon = fmt::weather_icon(e.hourly_forecast[i].code),
        .description = fmt::weather_description(e.hourly_forecast[i].code),
    };
  }

  std::array<WeatherDayForecast, Event::Weather::OnDay::COUNT> daily;
  for (size_t i = 0; i < Event::Weather::OnDay::COUNT; i++) {
    daily[i] = {
        .unix_seconds = e.daily_forecast[i].unix_seconds,
        .temperature_min = e.daily_forecast[i].temperature_min,
        .temperature_max = e.daily_forecast[i].temperature_max,
        .icon = fmt::weather_icon(e.daily_forecast[i].code),
        .description = fmt::weather_description(e.daily_forecast[i].code),
    };
  }

  Q_EMIT weatherChanged(summary, hourly, daily);
}
void UiModel::operator()(const Event::Network &e) {
  Q_EMIT networkSsidAndStrengthChanged(fmt::network_name(e.ssid, e.strength));
}
void UiModel::operator()(const Event::UploadSpeed &e) {
  Q_EMIT networkUploadSpeedChanged(fmt::network_speed(e.bytes_per_sec));
}
void UiModel::operator()(const Event::DownloadSpeed &e) {
  Q_EMIT networkDownloadSpeedChanged(fmt::network_speed(e.bytes_per_sec));
}
void UiModel::operator()(const Event::Tray::AppAdded &e) {
  Q_EMIT trayAppAdded(e.service, e.icon, e.items);
}
void UiModel::operator()(const Event::Tray::AppIconUpdated &e) {
  Q_EMIT trayAppIconUpdated(e.service, e.icon);
}
void UiModel::operator()(const Event::Tray::AppMenuUpdated &e) {
  Q_EMIT trayAppMenuUpdated(e.service, e.items);
}
void UiModel::operator()(const Event::Tray::AppRemoved &e) {
  Q_EMIT trayAppRemoved(e.service);
}
void UiModel::operator()(const Event::KbModToggled &e) {
  QString icon = e.enabled ? "" : "";
  QString text = fmt::kb_mod_label(e.kind, e.enabled);
  Q_EMIT kbModChanged(icon, text);
}
void UiModel::operator()(const Event::Sound &e) {
  Q_EMIT soundChanged(e.volume, fmt::sound(e.volume, e.muted));
}
void UiModel::operator()(
    [[maybe_unused]] const Event::ToggleSessionScreen &e_) {
  Q_EMIT sessionToggleRequested();
}
void UiModel::operator()([[maybe_unused]] const Event::Exit &e) {
  Q_EMIT exitRequested();
}
