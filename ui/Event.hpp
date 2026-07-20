#pragma once

#include "bindings.hpp"
#include <QIcon>
#include <QList>
#include <QString>
#include <array>

struct Event {
  struct Memory {
    double used;
    double total;

    Memory(const IoEvent::Memory_Body &data);
  };

  struct CPU {
    QList<uint8_t> usage_per_core;

    CPU(const IoEvent::CpuUsage_Body &data);
  };

  struct Time {
    QString now;

    Time(const IoEvent::Time_Body &data);
  };

  struct Language {
    QString lang;

    Language(const IoEvent::Language_Body &data);
  };

  struct Weather {
    struct OnHour {
      static constexpr size_t COUNT = HOURLY_WEATHER_FORECAST_LENGTH;
      using Array = std::array<OnHour, COUNT>;

      int64_t unix_seconds;
      float temperature;
      enum WeatherCode code;
    };

    struct OnDay {
      static constexpr size_t COUNT = DAILY_WEATHER_FORECAST_LENGTH;
      using Array = std::array<OnDay, COUNT>;

      int64_t unix_seconds;
      float temperature_min;
      float temperature_max;
      enum WeatherCode code;
    };

    float temperature;
    enum WeatherCode code;

    OnHour::Array hourly_forecast;
    OnDay::Array daily_forecast;

    Weather(const IoEvent::Weather_Body &data);
  };

  struct Network {
    QString ssid;
    uint8_t strength;

    Network(const IoEvent::NetworkSsidAndStrength_Body &data);
  };

  struct UploadSpeed {
    uint64_t bytes_per_sec;

    UploadSpeed(const IoEvent::UploadSpeed_Body &data);
  };

  struct DownloadSpeed {
    uint64_t bytes_per_sec;

    DownloadSpeed(const IoEvent::DownloadSpeed_Body &data);
  };

  struct Tray {
    struct MenuItem {
    public:
      struct Regular {
        int32_t id;
        uint32_t service;
        QString label;
      };

      struct Disabled {
        int32_t id;
        uint32_t service;
        QString label;
      };

      struct Checkbox {
        int32_t id;
        uint32_t service;
        QString label;
        bool checked;
      };

      struct Radio {
        int32_t id;
        uint32_t service;
        QString label;
        bool selected;
      };

      struct Nested {
        int32_t id;
        uint32_t service;
        QString label;
        QVector<MenuItem> children;
      };

      struct Section {
        QVector<MenuItem> children;
      };

      using Value =
          std::variant<Regular, Disabled, Checkbox, Radio, Nested, Section>;

      const Value &value() const;

      Value value_;
    };

    struct AppAdded {
      uint32_t service;
      QList<MenuItem> items;
      QIcon icon;

      AppAdded(const IoEvent::TrayAppAdded_Body &data);
    };

    struct AppIconUpdated {
      uint32_t service;
      QIcon icon;

      AppIconUpdated(const IoEvent::TrayAppIconUpdated_Body &data);
    };

    struct AppMenuUpdated {
      uint32_t service;
      QList<MenuItem> items;

      AppMenuUpdated(const IoEvent::TrayAppMenuUpdated_Body &data);
    };

    struct AppRemoved {
      uint32_t service;

      AppRemoved(const IoEvent::TrayAppRemoved_Body &data);
    };
  };

  struct KbModToggled {
    enum class Kind { CapsLock, NumLock };

    Kind kind;
    bool enabled;

    KbModToggled(const IoEvent::KbModToggled_Body &data);
  };

  struct Sound {
    uint8_t volume;
    bool muted;

    Sound(const IoEvent::Sound_Body &data);
  };

  struct ToggleSessionScreen {};
  struct Exit {};

  using Value =
      std::variant<Memory, CPU, Time, Language, Weather, Network, UploadSpeed,
                   DownloadSpeed, Tray::AppAdded, Tray::AppIconUpdated,
                   Tray::AppMenuUpdated, Tray::AppRemoved, KbModToggled, Sound,
                   ToggleSessionScreen, Exit>;

  static Event from(const IoEvent &nt);

  const Value &value() const;

private:
  Event(Value value);
  Value value_;
};
