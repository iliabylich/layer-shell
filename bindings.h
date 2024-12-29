#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum Drizzle {
  Light,
  Moderate,
  Dense,
} Drizzle;

typedef enum Fog {
  Normal,
  DepositingRime,
} Fog;

typedef enum FreezingDrizzle {
  Light,
  Dense,
} FreezingDrizzle;

typedef enum FreezingRain {
  Light,
  Heavy,
} FreezingRain;

typedef enum Rain {
  Slight,
  Moderate,
  Heavy,
} Rain;

typedef enum RainShowers {
  Slight,
  Moderate,
  Violent,
} RainShowers;

typedef enum SnowFall {
  Slight,
  Moderate,
  Heavy,
} SnowFall;

typedef enum SnowShowers {
  Slight,
  Heavy,
} SnowShowers;

typedef enum ThunderstormWithHail {
  Sight,
  Heavy,
} ThunderstormWithHail;

typedef struct CArray_usize {
  uintptr_t *ptr;
  uintptr_t len;
} CArray_usize;

typedef struct CString {
  char *ptr;
} CString;

typedef enum AppIcon_Tag {
  IconPath,
  IconName,
} AppIcon_Tag;

typedef struct AppIcon {
  AppIcon_Tag tag;
  union {
    struct {
      struct CString icon_path;
    };
    struct {
      struct CString icon_name;
    };
  };
} AppIcon;

typedef struct App {
  struct CString name;
  bool selected;
  struct AppIcon icon;
} App;

typedef struct CArray_App {
  struct App *ptr;
  uintptr_t len;
} CArray_App;

typedef enum WeatherCode_Tag {
  ClearSky,
  MainlyClear,
  PartlyCloudy,
  Overcast,
  Fog,
  Drizzle,
  FreezingDrizzle,
  Rain,
  FreezingRain,
  SnowFall,
  SnowGrains,
  RainShowers,
  SnowShowers,
  Thunderstorm,
  ThunderstormWithHail,
  Unknown,
} WeatherCode_Tag;

typedef struct WeatherCode {
  WeatherCode_Tag tag;
  union {
    struct {
      enum Fog fog;
    };
    struct {
      enum Drizzle drizzle;
    };
    struct {
      enum FreezingDrizzle freezing_drizzle;
    };
    struct {
      enum Rain rain;
    };
    struct {
      enum FreezingRain freezing_rain;
    };
    struct {
      enum SnowFall snow_fall;
    };
    struct {
      enum RainShowers rain_showers;
    };
    struct {
      enum SnowShowers snow_showers;
    };
    struct {
      enum ThunderstormWithHail thunderstorm_with_hail;
    };
  };
} WeatherCode;

typedef struct WeatherOnHour {
  struct CString hour;
  float temperature;
  struct WeatherCode code;
} WeatherOnHour;

typedef struct CArray_WeatherOnHour {
  struct WeatherOnHour *ptr;
  uintptr_t len;
} CArray_WeatherOnHour;

typedef struct WeatherOnDay {
  struct CString day;
  float temperature_min;
  float temperature_max;
  struct WeatherCode code;
} WeatherOnDay;

typedef struct CArray_WeatherOnDay {
  struct WeatherOnDay *ptr;
  uintptr_t len;
} CArray_WeatherOnDay;

typedef struct Network {
  struct CString iface;
  struct CString address;
} Network;

typedef struct CArray_Network {
  struct Network *ptr;
  uintptr_t len;
} CArray_Network;

typedef enum Event_Tag {
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
} Event_Tag;

typedef struct Memory_Body {
  double used;
  double total;
} Memory_Body;

typedef struct CpuUsage_Body {
  struct CArray_usize usage_per_core;
} CpuUsage_Body;

typedef struct Time_Body {
  struct CString date;
  struct CString time;
} Time_Body;

typedef struct Workspaces_Body {
  struct CArray_usize ids;
  uintptr_t active_id;
} Workspaces_Body;

typedef struct Language_Body {
  struct CString lang;
} Language_Body;

typedef struct AppList_Body {
  struct CArray_App apps;
} AppList_Body;

typedef struct Volume_Body {
  float volume;
} Volume_Body;

typedef struct CurrentWeather_Body {
  float temperature;
  struct WeatherCode code;
} CurrentWeather_Body;

typedef struct ForecastWeather_Body {
  struct CArray_WeatherOnHour hourly;
  struct CArray_WeatherOnDay daily;
} ForecastWeather_Body;

typedef struct WiFiStatus_Body {
  struct CString ssid;
  uint8_t strength;
} WiFiStatus_Body;

typedef struct NetworkList_Body {
  struct CArray_Network list;
} NetworkList_Body;

typedef struct Event {
  Event_Tag tag;
  union {
    Memory_Body memory;
    CpuUsage_Body cpu_usage;
    Time_Body time;
    Workspaces_Body workspaces;
    Language_Body language;
    AppList_Body app_list;
    Volume_Body volume;
    CurrentWeather_Body current_weather;
    ForecastWeather_Body forecast_weather;
    WiFiStatus_Body wi_fi_status;
    NetworkList_Body network_list;
  };
} Event;

typedef enum Command_Tag {
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
} Command_Tag;

typedef struct HyprlandGoToWorkspace_Body {
  uintptr_t idx;
} HyprlandGoToWorkspace_Body;

typedef struct AppListSetSearch_Body {
  const uint8_t *search;
} AppListSetSearch_Body;

typedef struct SetVolume_Body {
  double volume;
} SetVolume_Body;

typedef struct Command {
  Command_Tag tag;
  union {
    HyprlandGoToWorkspace_Body hyprland_go_to_workspace;
    AppListSetSearch_Body app_list_set_search;
    SetVolume_Body set_volume;
  };
} Command;

typedef struct CBytes {
  const uint8_t *content;
  uintptr_t len;
} CBytes;

extern const uint8_t *MAIN_CSS;

extern struct CBytes FOGGY;

extern struct CBytes QUESTION_MARK;

extern struct CBytes SUNNY;

extern struct CBytes PARTLY_CLOUDY;

extern struct CBytes RAINY;

extern struct CBytes THUNDERSTORM;

extern struct CBytes POWER;

extern struct CBytes SNOWY;

extern struct CBytes WIFI;

void subscribe(void (*f)(const struct Event*));

void init(void);

void spawn_thread(void);

void poll_events(void);

void publish(struct Command c);

void init_logger(void);
