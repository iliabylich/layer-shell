#ifndef BINDINGS_H
#define BINDINGS_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
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
} LAYER_SHELL_IO_WeatherCode;

typedef struct {
  size_t *ptr;
  size_t len;
} LAYER_SHELL_IO_CArray_usize;

typedef struct {
  char *ptr;
} LAYER_SHELL_IO_CString;

typedef enum {
  IconPath,
  IconName,
} LAYER_SHELL_IO_AppIcon_Tag;

typedef struct {
  LAYER_SHELL_IO_AppIcon_Tag tag;
  union {
    struct {
      LAYER_SHELL_IO_CString icon_path;
    };
    struct {
      LAYER_SHELL_IO_CString icon_name;
    };
  };
} LAYER_SHELL_IO_AppIcon;

typedef struct {
  LAYER_SHELL_IO_CString name;
  bool selected;
  LAYER_SHELL_IO_AppIcon icon;
} LAYER_SHELL_IO_App;

typedef struct {
  LAYER_SHELL_IO_App *ptr;
  size_t len;
} LAYER_SHELL_IO_CArray_App;

typedef struct {
  LAYER_SHELL_IO_CString hour;
  float temperature;
  LAYER_SHELL_IO_WeatherCode code;
} LAYER_SHELL_IO_WeatherOnHour;

typedef struct {
  LAYER_SHELL_IO_WeatherOnHour *ptr;
  size_t len;
} LAYER_SHELL_IO_CArray_WeatherOnHour;

typedef struct {
  LAYER_SHELL_IO_CString day;
  float temperature_min;
  float temperature_max;
  LAYER_SHELL_IO_WeatherCode code;
} LAYER_SHELL_IO_WeatherOnDay;

typedef struct {
  LAYER_SHELL_IO_WeatherOnDay *ptr;
  size_t len;
} LAYER_SHELL_IO_CArray_WeatherOnDay;

typedef struct {
  LAYER_SHELL_IO_CString iface;
  LAYER_SHELL_IO_CString address;
} LAYER_SHELL_IO_Network;

typedef struct {
  LAYER_SHELL_IO_Network *ptr;
  size_t len;
} LAYER_SHELL_IO_CArray_Network;

typedef enum {
  Memory,
  CpuUsage,
  Time,
  Workspaces,
  Language,
  AppList,
  Volume,
  CurrentWeather,
  ForecastWeather,
  WiFiStatus,
  NetworkList,
  ToggleLauncher,
  ToggleSessionScreen,
} LAYER_SHELL_IO_Event_Tag;

typedef struct {
  double used;
  double total;
} LAYER_SHELL_IO_Memory_Body;

typedef struct {
  LAYER_SHELL_IO_CArray_usize usage_per_core;
} LAYER_SHELL_IO_CpuUsage_Body;

typedef struct {
  LAYER_SHELL_IO_CString date;
  LAYER_SHELL_IO_CString time;
} LAYER_SHELL_IO_Time_Body;

typedef struct {
  LAYER_SHELL_IO_CArray_usize ids;
  size_t active_id;
} LAYER_SHELL_IO_Workspaces_Body;

typedef struct {
  LAYER_SHELL_IO_CString lang;
} LAYER_SHELL_IO_Language_Body;

typedef struct {
  LAYER_SHELL_IO_CArray_App apps;
} LAYER_SHELL_IO_AppList_Body;

typedef struct {
  float volume;
} LAYER_SHELL_IO_Volume_Body;

typedef struct {
  float temperature;
  LAYER_SHELL_IO_WeatherCode code;
} LAYER_SHELL_IO_CurrentWeather_Body;

typedef struct {
  LAYER_SHELL_IO_CArray_WeatherOnHour hourly;
  LAYER_SHELL_IO_CArray_WeatherOnDay daily;
} LAYER_SHELL_IO_ForecastWeather_Body;

typedef struct {
  LAYER_SHELL_IO_CString ssid;
  uint8_t strength;
} LAYER_SHELL_IO_WiFiStatus_Body;

typedef struct {
  LAYER_SHELL_IO_CArray_Network list;
} LAYER_SHELL_IO_NetworkList_Body;

typedef struct {
  LAYER_SHELL_IO_Event_Tag tag;
  union {
    LAYER_SHELL_IO_Memory_Body memory;
    LAYER_SHELL_IO_CpuUsage_Body cpu_usage;
    LAYER_SHELL_IO_Time_Body time;
    LAYER_SHELL_IO_Workspaces_Body workspaces;
    LAYER_SHELL_IO_Language_Body language;
    LAYER_SHELL_IO_AppList_Body app_list;
    LAYER_SHELL_IO_Volume_Body volume;
    LAYER_SHELL_IO_CurrentWeather_Body current_weather;
    LAYER_SHELL_IO_ForecastWeather_Body forecast_weather;
    LAYER_SHELL_IO_WiFiStatus_Body wi_fi_status;
    LAYER_SHELL_IO_NetworkList_Body network_list;
  };
} LAYER_SHELL_IO_Event;

typedef enum {
  HyprlandGoToWorkspace,
  AppListReset,
  AppListGoUp,
  AppListGoDown,
  AppListSetSearch,
  AppListExecSelected,
  SetVolume,
  Lock,
  Reboot,
  Shutdown,
  Logout,
  SpawnNetworkEditor,
  SpawnSystemMonitor,
} LAYER_SHELL_IO_Command_Tag;

typedef struct {
  size_t idx;
} LAYER_SHELL_IO_HyprlandGoToWorkspace_Body;

typedef struct {
  const uint8_t *search;
} LAYER_SHELL_IO_AppListSetSearch_Body;

typedef struct {
  float volume;
} LAYER_SHELL_IO_SetVolume_Body;

typedef struct {
  LAYER_SHELL_IO_Command_Tag tag;
  union {
    LAYER_SHELL_IO_HyprlandGoToWorkspace_Body hyprland_go_to_workspace;
    LAYER_SHELL_IO_AppListSetSearch_Body app_list_set_search;
    LAYER_SHELL_IO_SetVolume_Body set_volume;
  };
} LAYER_SHELL_IO_Command;

typedef struct {
  const uint8_t *content;
  size_t len;
} LAYER_SHELL_IO_CBytes;

extern LAYER_SHELL_IO_CBytes FOGGY_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes QUESTION_MARK_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes SUNNY_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes PARTLY_CLOUDY_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes RAINY_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes THUNDERSTORM_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes POWER_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes SNOWY_ICON_BYTES;

extern LAYER_SHELL_IO_CBytes WIFI_ICON_BYTES;

void layer_shell_io_subscribe(void (*f)(const LAYER_SHELL_IO_Event*));

void layer_shell_io_init(void);

void layer_shell_io_spawn_thread(void);

void layer_shell_io_poll_events(void);

void layer_shell_io_publish(LAYER_SHELL_IO_Command command);

void layer_shell_io_init_logger(void);

LAYER_SHELL_IO_CString layer_shell_io_main_css(void);

void layer_shell_io_on_sigusr1(void);

#endif  /* BINDINGS_H */
