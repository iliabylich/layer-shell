#include "bindings.h"
#include "ui/bluetooth.h"
#include "ui/change_theme.h"
#include "ui/clock.h"
#include "ui/cpu.h"
#include "ui/css.h"
#include "ui/language.h"
#include "ui/logger.h"
#include "ui/memory.h"
#include "ui/network.h"
#include "ui/ping_window.h"
#include "ui/power.h"
#include "ui/session_window.h"
#include "ui/sound_window.h"
#include "ui/terminal.h"
#include "ui/terminal_window.h"
#include "ui/top_bar.h"
#include "ui/tray.h"
#include "ui/weather.h"
#include "ui/weather_window.h"
#include "ui/workspaces.h"
#include <gtk/gtk.h>

LOGGER("main.c", 0)

GtkApplication *app;

GtkWidget *top_bar;
GtkWidget *weather_window;
GtkWidget *terminal_window;
GtkWidget *ping_window;
GtkWidget *session_window;
GtkWidget *sound_window;

GtkWidget *workspaces;
GtkWidget *change_theme;
GtkWidget *tray;
GtkWidget *weather;
GtkWidget *terminal;
GtkWidget *language;
GtkWidget *cpu;
GtkWidget *memory;
GtkWidget *network;
GtkWidget *bluetooth;
GtkWidget *clock_;
GtkWidget *power;

void remove_window(GtkWidget **win) {
  gtk_application_remove_window(app, GTK_WINDOW(*win));
  g_clear_pointer(win, g_object_unref);
}

int poll_events(void) {
  IO_CArray_Event events = io_poll_events();
  bool keep_processing = true;
  for (size_t i = 0; i < events.len && keep_processing; i++) {
    IO_Event event = events.ptr[i];
    switch (event.tag) {
    case IO_Event_Workspaces: {
      workspaces_refresh(WORKSPACES(workspaces), event.workspaces);
      break;
    }
    case IO_Event_ReloadStyles: {
      css_reload();
      break;
    }
    case IO_Event_TrayAppAdded: {
      tray_add_app(TRAY(tray), event.tray_app_added);
      break;
    }
    case IO_Event_TrayAppRemoved: {
      tray_remove_app(TRAY(tray), event.tray_app_removed);
      break;
    }
    case IO_Event_TrayAppIconUpdated: {
      tray_update_icon(TRAY(tray), event.tray_app_icon_updated);
      break;
    }
    case IO_Event_TrayAppMenuUpdated: {
      tray_update_menu(TRAY(tray), event.tray_app_menu_updated);
      break;
    }
    case IO_Event_CurrentWeather: {
      weather_refresh(WEATHER(weather), event.current_weather);
      break;
    }
    case IO_Event_HourlyWeatherForecast: {
      weather_window_refresh_hourly_forecast(WEATHER_WINDOW(weather_window),
                                             event.hourly_weather_forecast);
      break;
    }
    case IO_Event_DailyWeatherForecast: {
      weather_window_refresh_daily_forecast(WEATHER_WINDOW(weather_window),
                                            event.daily_weather_forecast);
      break;
    }
    case IO_Event_Language: {
      language_refresh(LANGUAGE(language), event.language);
      break;
    }
    case IO_Event_CpuUsage: {
      cpu_refresh(CPU(cpu), event.cpu_usage);
      break;
    }
    case IO_Event_Memory: {
      memory_refresh(MEMORY(memory), event.memory);
      break;
    }
    case IO_Event_WifiStatus: {
      network_refresh_wifi_status(NETWORK(network), event.wifi_status);
      break;
    }
    case IO_Event_DownloadSpeed: {
      network_refresh_download_speed(NETWORK(network), event.download_speed);
      break;
    }
    case IO_Event_UploadSpeed: {
      network_refresh_upload_speed(NETWORK(network), event.upload_speed);
      break;
    }
    case IO_Event_NetworkList: {
      network_refresh_network_list(NETWORK(network), event.network_list);
      break;
    }
    case IO_Event_Clock: {
      clock_refresh(CLOCK(clock_), event.clock);
      break;
    }
    case IO_Event_ToggleSessionScreen: {
      session_window_toggle(SESSION_WINDOW(session_window));
      break;
    }
    case IO_Event_InitialSound: {
      sound_window_set_initial_sound(SOUND_WINDOW(sound_window),
                                     event.initial_sound);
      break;
    }
    case IO_Event_VolumeChanged: {
      sound_window_refresh_volume(SOUND_WINDOW(sound_window),
                                  event.volume_changed);
      break;
    }
    case IO_Event_MuteChanged: {
      sound_window_refresh_mute(SOUND_WINDOW(sound_window), event.mute_changed);
      break;
    }
    case IO_Event_Exit: {
      LOG("Received exit...");
      io_finalize();
      LOG("Removing windows...");
      remove_window(&top_bar);
      remove_window(&weather_window);
      remove_window(&terminal_window);
      remove_window(&ping_window);
      remove_window(&session_window);
      g_application_quit(G_APPLICATION(app));
      LOG("Quit done.");
      keep_processing = false;
      break;
    }
    }
  }
  io_drop_events(events);
  return 1;
}

static void on_workspace_switched(Workspaces *, guint idx) {
  io_hyprland_go_to_workspace(idx);
}

static void on_theme_change_clicked() { io_change_theme(); }

static void on_tray_triggered(Tray *, const char *uuid) {
  io_trigger_tray(uuid);
}

static void on_weather_clicked() {
  weather_window_toggle(WEATHER_WINDOW(weather_window));
}

static void on_terminal_clicked() {
  terminal_window_toggle(TERMINAL_WINDOW(terminal_window));
}

static void on_memory_clicked() { io_spawn_system_monitor(); }

static void on_network_settings_clicked() { io_spawn_wifi_editor(); }

static void on_network_ping_clicked() {
  ping_window_toggle(PING_WINDOW(ping_window));
}

static void on_network_address_clicked(Network *, const char *ip) {
  GdkDisplay *display = gdk_display_get_default();
  GdkClipboard *clipboard = gdk_display_get_clipboard(display);
  gdk_clipboard_set_text(clipboard, ip);

  char format[100];
  sprintf(format, "Copied %s", ip);
  GNotification *notification = g_notification_new(format);
  g_application_send_notification(G_APPLICATION(app), NULL, notification);
}

static void on_bluetooth_clicked() { io_spawn_bluetooh_editor(); }

static void on_power_clicked() {
  session_window_toggle(SESSION_WINDOW(session_window));
}

static void on_lock_clicked() { io_lock(); }
static void on_reboot_clicked() { io_reboot(); }
static void on_shutdown_clicked() { io_shutdown(); }
static void on_logout_clicked() { io_logout(); }

static void on_app_activate() {
  top_bar = top_bar_new(app);

#define CONNECT(widget, signal, callback)                                      \
  g_signal_connect(widget, signal, G_CALLBACK(callback), NULL)

  workspaces = workspaces_new();
  CONNECT(workspaces, "switched", on_workspace_switched);
  top_bar_push_left(TOP_BAR(top_bar), workspaces);

  change_theme = change_theme_new();
  CONNECT(change_theme, "clicked", on_theme_change_clicked);
  top_bar_push_left(TOP_BAR(top_bar), change_theme);

  tray = tray_new();
  CONNECT(tray, "triggered", on_tray_triggered);
  top_bar_push_right(TOP_BAR(top_bar), tray);

  weather = weather_new();
  CONNECT(weather, "clicked", on_weather_clicked);
  top_bar_push_right(TOP_BAR(top_bar), weather);

  terminal = terminal_new();
  CONNECT(terminal, "clicked", on_terminal_clicked);
  top_bar_push_right(TOP_BAR(top_bar), terminal);

  language = language_new();
  top_bar_push_right(TOP_BAR(top_bar), language);

  cpu = cpu_new();
  top_bar_push_right(TOP_BAR(top_bar), cpu);

  memory = memory_new();
  CONNECT(memory, "clicked", on_memory_clicked);
  top_bar_push_right(TOP_BAR(top_bar), memory);

  network = network_new();
  CONNECT(network, "clicked-settings", on_network_settings_clicked);
  CONNECT(network, "clicked-ping", on_network_ping_clicked);
  CONNECT(network, "clicked-address", on_network_address_clicked);
  top_bar_push_right(TOP_BAR(top_bar), network);

  bluetooth = bluetooth_new();
  CONNECT(bluetooth, "clicked", on_bluetooth_clicked);
  top_bar_push_right(TOP_BAR(top_bar), bluetooth);

  clock_ = clock_new();
  top_bar_push_right(TOP_BAR(top_bar), clock_);

  power = power_new();
  CONNECT(power, "clicked", on_power_clicked);
  top_bar_push_right(TOP_BAR(top_bar), power);

  weather_window = weather_window_new(app);

  terminal_window = terminal_window_new(app);

  ping_window = ping_window_new(app);

  session_window = session_window_new(app);
  CONNECT(session_window, "clicked-lock", on_lock_clicked);
  CONNECT(session_window, "clicked-shutdown", on_shutdown_clicked);
  CONNECT(session_window, "clicked-reboot", on_reboot_clicked);
  CONNECT(session_window, "clicked-logout", on_logout_clicked);

  sound_window = sound_window_new(app);

#undef CONNECT

  gtk_window_present(GTK_WINDOW(top_bar));

  g_timeout_add(50, G_SOURCE_FUNC(poll_events), NULL);

  io_spawn_thread();
}

int main(int argc, char **argv) {
  setenv("GSK_RENDERER", "cairo", true);
  io_init();

  app = gtk_application_new("org.me.LayerShell", G_APPLICATION_DEFAULT_FLAGS);
  g_signal_connect(app, "activate", on_app_activate, NULL);
  g_signal_connect(app, "startup", css_load, NULL);
  g_application_run(G_APPLICATION(app), argc, argv);
  g_object_unref(app);

  return 0;
}
