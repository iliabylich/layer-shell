#ifndef BINDINGS_H
#define BINDINGS_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  IO_WeatherCode_ClearSky,
  IO_WeatherCode_MainlyClear,
  IO_WeatherCode_PartlyCloudy,
  IO_WeatherCode_Overcast,
  IO_WeatherCode_FogNormal,
  IO_WeatherCode_FogDepositingRime,
  IO_WeatherCode_DrizzleLight,
  IO_WeatherCode_DrizzleModerate,
  IO_WeatherCode_DrizzleDense,
  IO_WeatherCode_FreezingDrizzleLight,
  IO_WeatherCode_FreezingDrizzleDense,
  IO_WeatherCode_RainSlight,
  IO_WeatherCode_RainModerate,
  IO_WeatherCode_RainHeavy,
  IO_WeatherCode_FreezingRainLight,
  IO_WeatherCode_FreezingRainHeavy,
  IO_WeatherCode_SnowFallSlight,
  IO_WeatherCode_SnowFallModerate,
  IO_WeatherCode_SnowFallHeavy,
  IO_WeatherCode_SnowGrains,
  IO_WeatherCode_RainShowersSlight,
  IO_WeatherCode_RainShowersModerate,
  IO_WeatherCode_RainShowersViolent,
  IO_WeatherCode_SnowShowersSlight,
  IO_WeatherCode_SnowShowersHeavy,
  IO_WeatherCode_Thunderstorm,
  IO_WeatherCode_ThunderstormWithHailSight,
  IO_WeatherCode_ThunderstormWithHailHeavy,
  IO_WeatherCode_Unknown,
} IO_WeatherCode;

typedef struct {
  size_t *ptr;
  size_t len;
} IO_CArray_usize;

typedef char *IO_CString;

typedef enum {
  IO_AppIcon_IconPath,
  IO_AppIcon_IconName,
} IO_AppIcon_Tag;

typedef struct {
  IO_AppIcon_Tag tag;
  union {
    struct {
      IO_CString icon_path;
    };
    struct {
      IO_CString icon_name;
    };
  };
} IO_AppIcon;

typedef struct {
  IO_CString name;
  bool selected;
  IO_AppIcon icon;
} IO_App;

typedef struct {
  IO_App *ptr;
  size_t len;
} IO_CArray_App;

typedef struct {
  IO_CString hour;
  float temperature;
  IO_WeatherCode code;
} IO_WeatherOnHour;

typedef struct {
  IO_WeatherOnHour *ptr;
  size_t len;
} IO_CArray_WeatherOnHour;

typedef struct {
  IO_CString day;
  float temperature_min;
  float temperature_max;
  IO_WeatherCode code;
} IO_WeatherOnDay;

typedef struct {
  IO_WeatherOnDay *ptr;
  size_t len;
} IO_CArray_WeatherOnDay;

typedef struct {
  IO_CString ssid;
  uint8_t strength;
} IO_WifiStatus;

typedef enum {
  IO_COption_WifiStatus_None_WifiStatus,
  IO_COption_WifiStatus_Some_WifiStatus,
} IO_COption_WifiStatus_Tag;

typedef struct {
  IO_COption_WifiStatus_Tag tag;
  union {
    struct {
      IO_WifiStatus some;
    };
  };
} IO_COption_WifiStatus;

typedef struct {
  IO_CString iface;
  IO_CString address;
} IO_Network;

typedef struct {
  IO_Network *ptr;
  size_t len;
} IO_CArray_Network;

typedef struct {
  IO_CString label;
  IO_CString uuid;
} IO_TrayItem;

typedef struct {
  IO_TrayItem *ptr;
  size_t len;
} IO_CArray_TrayItem;

typedef struct {
  uint8_t *ptr;
  size_t len;
} IO_CArray_u8;

typedef enum {
  IO_TrayIcon_Path,
  IO_TrayIcon_Name,
  IO_TrayIcon_PixmapVariant,
  IO_TrayIcon_None,
} IO_TrayIcon_Tag;

typedef struct {
  IO_CString path;
} IO_TrayIcon_IO_Path_Body;

typedef struct {
  IO_CString name;
} IO_TrayIcon_IO_Name_Body;

typedef struct {
  uint32_t w;
  uint32_t h;
  IO_CArray_u8 bytes;
} IO_TrayIcon_IO_PixmapVariant_Body;

typedef struct {
  IO_TrayIcon_Tag tag;
  union {
    IO_TrayIcon_IO_Path_Body path;
    IO_TrayIcon_IO_Name_Body name;
    IO_TrayIcon_IO_PixmapVariant_Body pixmap_variant;
  };
} IO_TrayIcon;

typedef struct {
  IO_CArray_TrayItem items;
  IO_TrayIcon icon;
} IO_TrayApp;

typedef struct {
  IO_TrayApp *ptr;
  size_t len;
} IO_CArray_TrayApp;

typedef enum {
  IO_Event_Memory,
  IO_Event_CpuUsage,
  IO_Event_Time,
  IO_Event_Workspaces,
  IO_Event_Language,
  IO_Event_AppList,
  IO_Event_Volume,
  IO_Event_Mute,
  IO_Event_CurrentWeather,
  IO_Event_ForecastWeather,
  IO_Event_WifiStatus,
  IO_Event_NetworkSpeed,
  IO_Event_NetworkList,
  IO_Event_Tray,
  IO_Event_ToggleLauncher,
  IO_Event_ToggleSessionScreen,
} IO_Event_Tag;

typedef struct {
  double used;
  double total;
} IO_Event_IO_Memory_Body;

typedef struct {
  IO_CArray_usize usage_per_core;
} IO_Event_IO_CpuUsage_Body;

typedef struct {
  IO_CString date;
  IO_CString time;
} IO_Event_IO_Time_Body;

typedef struct {
  IO_CArray_usize ids;
  size_t active_id;
} IO_Event_IO_Workspaces_Body;

typedef struct {
  IO_CString lang;
} IO_Event_IO_Language_Body;

typedef struct {
  IO_CArray_App apps;
} IO_Event_IO_AppList_Body;

typedef struct {
  float volume;
} IO_Event_IO_Volume_Body;

typedef struct {
  bool muted;
} IO_Event_IO_Mute_Body;

typedef struct {
  float temperature;
  IO_WeatherCode code;
} IO_Event_IO_CurrentWeather_Body;

typedef struct {
  IO_CArray_WeatherOnHour hourly;
  IO_CArray_WeatherOnDay daily;
} IO_Event_IO_ForecastWeather_Body;

typedef struct {
  IO_COption_WifiStatus wifi_status;
} IO_Event_IO_WifiStatus_Body;

typedef struct {
  IO_CString upload_speed;
  IO_CString download_speed;
} IO_Event_IO_NetworkSpeed_Body;

typedef struct {
  IO_CArray_Network list;
} IO_Event_IO_NetworkList_Body;

typedef struct {
  IO_CArray_TrayApp list;
} IO_Event_IO_Tray_Body;

typedef struct {
  IO_Event_Tag tag;
  union {
    IO_Event_IO_Memory_Body memory;
    IO_Event_IO_CpuUsage_Body cpu_usage;
    IO_Event_IO_Time_Body time;
    IO_Event_IO_Workspaces_Body workspaces;
    IO_Event_IO_Language_Body language;
    IO_Event_IO_AppList_Body app_list;
    IO_Event_IO_Volume_Body volume;
    IO_Event_IO_Mute_Body mute;
    IO_Event_IO_CurrentWeather_Body current_weather;
    IO_Event_IO_ForecastWeather_Body forecast_weather;
    IO_Event_IO_WifiStatus_Body wifi_status;
    IO_Event_IO_NetworkSpeed_Body network_speed;
    IO_Event_IO_NetworkList_Body network_list;
    IO_Event_IO_Tray_Body tray;
  };
} IO_Event;

typedef enum {
  IO_Command_HyprlandGoToWorkspace,
  IO_Command_AppListReset,
  IO_Command_AppListGoUp,
  IO_Command_AppListGoDown,
  IO_Command_AppListSetSearch,
  IO_Command_AppListExecSelected,
  IO_Command_SetVolume,
  IO_Command_SetMuted,
  IO_Command_Lock,
  IO_Command_Reboot,
  IO_Command_Shutdown,
  IO_Command_Logout,
  IO_Command_TriggerTray,
  IO_Command_SpawnNetworkEditor,
  IO_Command_SpawnSystemMonitor,
} IO_Command_Tag;

typedef struct {
  size_t idx;
} IO_Command_IO_HyprlandGoToWorkspace_Body;

typedef struct {
  const uint8_t *search;
} IO_Command_IO_AppListSetSearch_Body;

typedef struct {
  float volume;
} IO_Command_IO_SetVolume_Body;

typedef struct {
  bool muted;
} IO_Command_IO_SetMuted_Body;

typedef struct {
  const uint8_t *uuid;
} IO_Command_IO_TriggerTray_Body;

typedef struct {
  IO_Command_Tag tag;
  union {
    IO_Command_IO_HyprlandGoToWorkspace_Body hyprland_go_to_workspace;
    IO_Command_IO_AppListSetSearch_Body app_list_set_search;
    IO_Command_IO_SetVolume_Body set_volume;
    IO_Command_IO_SetMuted_Body set_muted;
    IO_Command_IO_TriggerTray_Body trigger_tray;
  };
} IO_Command;

void layer_shell_io_subscribe(void (*f)(const IO_Event*));

void layer_shell_io_init(void);

void layer_shell_io_spawn_thread(void);

void layer_shell_io_poll_events(void);

void layer_shell_io_publish(IO_Command command);

void layer_shell_io_on_sigusr1(void);

#endif  /* BINDINGS_H */
