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

    Memory(const IO_Event::IO_Memory_Body &data);
  };

  struct CPU {
    QList<uint8_t> usage_per_core;

    CPU(const IO_Event::IO_CpuUsage_Body &data);
  };

  struct Time {
    QString now;

    Time(const IO_Event::IO_Time_Body &data);
  };

  struct Language {
    QString lang;

    Language(const IO_Event::IO_Language_Body &data);
  };

  struct Weather {
    struct OnHour {
      static constexpr size_t COUNT = IO_HOURLY_WEATHER_FORECAST_LENGTH;
      using Array = std::array<OnHour, COUNT>;

      int64_t unix_seconds;
      float temperature;
      enum IO_WeatherCode code;
    };

    struct OnDay {
      static constexpr size_t COUNT = IO_DAILY_WEATHER_FORECAST_LENGTH;
      using Array = std::array<OnDay, COUNT>;

      int64_t unix_seconds;
      float temperature_min;
      float temperature_max;
      enum IO_WeatherCode code;
    };

    float temperature;
    enum IO_WeatherCode code;

    OnHour::Array hourly_forecast;
    OnDay::Array daily_forecast;

    Weather(const IO_Event::IO_Weather_Body &data);
  };

  struct Network {
    QString ssid;
    uint8_t strength;

    Network(const IO_Event::IO_NetworkSsidAndStrength_Body &data);
  };

  struct UploadSpeed {
    uint64_t bytes_per_sec;

    UploadSpeed(const IO_Event::IO_UploadSpeed_Body &data);
  };

  struct DownloadSpeed {
    uint64_t bytes_per_sec;

    DownloadSpeed(const IO_Event::IO_DownloadSpeed_Body &data);
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

      AppAdded(const IO_Event::IO_TrayAppAdded_Body &data);
    };

    struct AppIconUpdated {
      uint32_t service;
      QIcon icon;

      AppIconUpdated(const IO_Event::IO_TrayAppIconUpdated_Body &data);
    };

    struct AppMenuUpdated {
      uint32_t service;
      QList<MenuItem> items;

      AppMenuUpdated(const IO_Event::IO_TrayAppMenuUpdated_Body &data);
    };

    struct AppRemoved {
      uint32_t service;

      AppRemoved(const IO_Event::IO_TrayAppRemoved_Body &data);
    };
  };

  struct KbModToggled {
    enum class Kind { CapsLock, NumLock };

    Kind kind;
    bool enabled;

    KbModToggled(const IO_Event::IO_KbModToggled_Body &data);
  };

  struct Sound {
    uint8_t volume;
    bool muted;

    Sound(const IO_Event::IO_Sound_Body &data);
  };

  struct ToggleSessionScreen {};
  struct Exit {};

  using Value =
      std::variant<Memory, CPU, Time, Language, Weather, Network, UploadSpeed,
                   DownloadSpeed, Tray::AppAdded, Tray::AppIconUpdated,
                   Tray::AppMenuUpdated, Tray::AppRemoved, KbModToggled, Sound,
                   ToggleSessionScreen, Exit>;

  static Event from(const IO_Event &event);

  const Value &value() const;

private:
  Event(Value value);
  Value value_;
};
