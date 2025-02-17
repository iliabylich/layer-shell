#pragma once

#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>

namespace layer_shell_io {

enum class WeatherCode {
  ClearSky,
  MainlyClear,
  PartlyCloudy,
  Overcast,
  FogNormal,
  FogDepositingRime,
  DrizzleLight,
  DrizzleModerate,
  DrizzleDense,
  FreezingDrizzleLight,
  FreezingDrizzleDense,
  RainSlight,
  RainModerate,
  RainHeavy,
  FreezingRainLight,
  FreezingRainHeavy,
  SnowFallSlight,
  SnowFallModerate,
  SnowFallHeavy,
  SnowGrains,
  RainShowersSlight,
  RainShowersModerate,
  RainShowersViolent,
  SnowShowersSlight,
  SnowShowersHeavy,
  Thunderstorm,
  ThunderstormWithHailSight,
  ThunderstormWithHailHeavy,
  Unknown,
};

template<typename T>
struct CArray {
  T *ptr;
  size_t len;
};

using CString = char*;

struct AppIcon {
  enum class Tag {
    IconPath,
    IconName,
  };

  struct IconPath_Body {
    CString _0;
  };

  struct IconName_Body {
    CString _0;
  };

  Tag tag;
  union {
    IconPath_Body icon_path;
    IconName_Body icon_name;
  };
};

struct App {
  CString name;
  bool selected;
  AppIcon icon;
};

struct WeatherOnHour {
  CString hour;
  float temperature;
  WeatherCode code;
};

struct WeatherOnDay {
  CString day;
  float temperature_min;
  float temperature_max;
  WeatherCode code;
};

struct WifiStatus {
  CString ssid;
  uint8_t strength;
};

template<typename T>
struct COption {
  enum class Tag {
    None,
    Some,
  };

  struct Some_Body {
    T _0;
  };

  Tag tag;
  union {
    Some_Body some;
  };
};

struct Network {
  CString iface;
  CString address;
};

struct TrayItem {
  int32_t id;
  CString uuid;
  CString type_;
  CString label;
  bool enabled;
  bool visible;
  CString icon_name;
  CString icon_data;
  CString toggle_type;
  int64_t toggle_state;
  CString children_display;
  CArray<TrayItem> children;
};

struct TrayIcon {
  enum class Tag {
    Path,
    Name,
    PixmapVariant,
    None,
  };

  struct Path_Body {
    CString path;
  };

  struct Name_Body {
    CString name;
  };

  struct PixmapVariant_Body {
    uint32_t w;
    uint32_t h;
    CArray<uint8_t> bytes;
  };

  Tag tag;
  union {
    Path_Body path;
    Name_Body name;
    PixmapVariant_Body pixmap_variant;
  };
};

struct TrayApp {
  TrayItem root_item;
  TrayIcon icon;
};

struct Event {
  enum class Tag {
    Memory,
    CpuUsage,
    Time,
    Workspaces,
    Language,
    Launcher,
    Volume,
    Mute,
    CurrentWeather,
    ForecastWeather,
    WifiStatus,
    NetworkSpeed,
    NetworkList,
    Tray,
    ToggleLauncher,
    ToggleSessionScreen,
  };

  struct Memory_Body {
    double used;
    double total;
  };

  struct CpuUsage_Body {
    CArray<size_t> usage_per_core;
  };

  struct Time_Body {
    CString date;
    CString time;
  };

  struct Workspaces_Body {
    CArray<size_t> ids;
    size_t active_id;
  };

  struct Language_Body {
    CString lang;
  };

  struct Launcher_Body {
    CArray<App> apps;
  };

  struct Volume_Body {
    uint32_t volume;
    bool muted;
  };

  struct Mute_Body {
    bool muted;
  };

  struct CurrentWeather_Body {
    float temperature;
    WeatherCode code;
  };

  struct ForecastWeather_Body {
    CArray<WeatherOnHour> hourly;
    CArray<WeatherOnDay> daily;
  };

  struct WifiStatus_Body {
    COption<WifiStatus> wifi_status;
  };

  struct NetworkSpeed_Body {
    CString upload_speed;
    CString download_speed;
  };

  struct NetworkList_Body {
    CArray<Network> list;
  };

  struct Tray_Body {
    CArray<TrayApp> list;
  };

  Tag tag;
  union {
    Memory_Body memory;
    CpuUsage_Body cpu_usage;
    Time_Body time;
    Workspaces_Body workspaces;
    Language_Body language;
    Launcher_Body launcher;
    Volume_Body volume;
    Mute_Body mute;
    CurrentWeather_Body current_weather;
    ForecastWeather_Body forecast_weather;
    WifiStatus_Body wifi_status;
    NetworkSpeed_Body network_speed;
    NetworkList_Body network_list;
    Tray_Body tray;
  };
};

extern "C" {

void *layer_shell_io_init();

void layer_shell_io_subscribe(void (*f)(const Event*, void*), void *data, void *ctx);

void layer_shell_io_spawn_thread(void *ctx);

void layer_shell_io_poll_events(void *ctx);

void layer_shell_io_hyprland_go_to_workspace(size_t idx, void *ctx);

void layer_shell_io_launcher_reset(void *ctx);

void layer_shell_io_launcher_go_up(void *ctx);

void layer_shell_io_launcher_go_down(void *ctx);

void layer_shell_io_launcher_set_search(const char *search, void *ctx);

void layer_shell_io_launcher_exec_selected(void *ctx);

void layer_shell_io_lock(void *ctx);

void layer_shell_io_reboot(void *ctx);

void layer_shell_io_shutdown(void *ctx);

void layer_shell_io_logout(void *ctx);

void layer_shell_io_trigger_tray(const char *uuid, void *ctx);

void layer_shell_io_spawn_network_editor(void *ctx);

void layer_shell_io_spawn_system_monitor(void *ctx);

}  // extern "C"

}  // namespace layer_shell_io
