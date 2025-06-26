#include "bindings.h"
#include "ui/include/builder.h"
#include "ui/include/css.h"
#include "ui/include/htop.h"
#include "ui/include/ping.h"
#include "ui/include/session.h"
#include "ui/include/top_bar.h"
#include "ui/include/top_bar/change_theme.h"
#include "ui/include/top_bar/clock.h"
#include "ui/include/top_bar/cpu.h"
#include "ui/include/top_bar/htop_button.h"
#include "ui/include/top_bar/language.h"
#include "ui/include/top_bar/memory.h"
#include "ui/include/top_bar/network.h"
#include "ui/include/top_bar/power.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/weather_button.h"
#include "ui/include/top_bar/workspaces.h"
#include "ui/include/weather.h"
#include <gtk/gtk.h>

GtkApplication *app;

GtkWidget *top_bar;
GtkWidget *weather;
GtkWidget *htop;
GtkWidget *ping;
GtkWidget *session;

GtkWidget *workspaces;
GtkWidget *change_theme;
GtkWidget *tray;
GtkWidget *weather_button;
GtkWidget *htop_button;
GtkWidget *language;
GtkWidget *cpu;
GtkWidget *memory;
GtkWidget *network;
GtkWidget *clock_;
GtkWidget *power;

static void drop_window(GtkWindow *win) {
  gtk_application_remove_window(app, GTK_WINDOW(win));
  g_object_unref(G_OBJECT(win));
}

int poll_events(void) {
  IO_CArray_Event events = io_poll_events();
  bool keep_processing = true;
  for (size_t i = 0; i < events.len && keep_processing; i++) {
    IO_Event event = events.ptr[i];
    switch (event.tag) {
    case IO_Event_Workspaces: {
      workspaces_refresh(workspaces, event.workspaces);
      break;
    }
    case IO_Event_ReloadStyles: {
      css_reload();
      break;
    }
    case IO_Event_TrayAppUpdated: {
      tray_update_app(tray, event.tray_app_updated);
      break;
    }
    case IO_Event_TrayAppRemoved: {
      tray_remove_app(tray, event.tray_app_removed);
      break;
    }
    case IO_Event_CurrentWeather: {
      weather_button_refresh(weather_button, event.current_weather);
      break;
    }
    case IO_Event_HourlyWeatherForecast: {
      weather_refresh_hourly_forecast(weather, event.hourly_weather_forecast);
      break;
    }
    case IO_Event_DailyWeatherForecast: {
      weather_refresh_daily_forecast(weather, event.daily_weather_forecast);
      break;
    }
    case IO_Event_Language: {
      language_refresh(language, event.language.lang);
      break;
    }
    case IO_Event_CpuUsage: {
      cpu_refresh(cpu, event.cpu_usage);
      break;
    }
    case IO_Event_Memory: {
      memory_refresh(memory, event.memory);
      break;
    }
    case IO_Event_WifiStatus: {
      network_refresh_wifi_status(network, event.wifi_status);
      break;
    }
    case IO_Event_DownloadSpeed: {
      network_refresh_download_speed(network, event.download_speed);
      break;
    }
    case IO_Event_UploadSpeed: {
      network_refresh_upload_speed(network, event.upload_speed);
      break;
    }
    case IO_Event_NetworkList: {
      network_refresh_network_list(network, event.network_list);
      break;
    }
    case IO_Event_Clock: {
      clock_refresh(clock_, event.clock);
      break;
    }
    case IO_Event_ToggleSessionScreen: {
      session_toggle(session);
      break;
    }
    case IO_Event_Exit: {
      fprintf(stderr, "[UI] Received exit...\n");
      io_finalize();
      fprintf(stderr, "[UI] Removing windows...\n");
      drop_window(GTK_WINDOW(top_bar));
      drop_window(GTK_WINDOW(weather));
      drop_window(GTK_WINDOW(htop));
      drop_window(GTK_WINDOW(ping));
      drop_window(GTK_WINDOW(session));
      g_application_quit(G_APPLICATION(app));
      fprintf(stderr, "[UI] Quit done.\n");
      keep_processing = false;
      break;
    }
    }
  }
  io_drop_events(events);
  return 1;
}

static void on_workspace_change_clicked(size_t idx) {
  io_hyprland_go_to_workspace(idx);
}

static void on_theme_change_clicked() { io_change_theme(); }

static void on_tray_triggered(const uint8_t *uuid) { io_trigger_tray(uuid); }

static void on_weather_button_clicked() { weather_toggle(weather); }

static void on_htop_button_clicked() { htop_toggle(htop); }

static void on_memory_clicked() { io_spawn_system_monitor(); }

static void on_network_settings_clicked() { io_spawn_network_editor(); }

static void on_network_ping_clicked() { ping_toggle(ping); }

static void on_nework_address_clicked(const char *ip) {
  GdkDisplay *display = gdk_display_get_default();
  GdkClipboard *clipboard = gdk_display_get_clipboard(display);
  gdk_clipboard_set_text(clipboard, ip);

  char format[100];
  sprintf(format, "Copied %s", ip);
  GNotification *notification = g_notification_new(format);
  g_application_send_notification(G_APPLICATION(app), NULL, notification);
}

static void on_power_clicked() { session_toggle(session); }

static void on_lock_clicked() { io_lock(); }
static void on_reboot_clicked() { io_reboot(); }
static void on_shutdown_clicked() { io_shutdown(); }
static void on_logout_clicked() { io_logout(); }

static void on_app_activate() {
  init_builders();

  top_bar = top_bar_init(app);
  weather = weather_init(app);
  htop = htop_init(app);
  ping = ping_init(app);
  session = session_init(app, on_lock_clicked, on_reboot_clicked,
                         on_shutdown_clicked, on_logout_clicked);

  workspaces = workspaces_init(on_workspace_change_clicked);
  change_theme = change_theme_init(on_theme_change_clicked);

  tray = tray_init(on_tray_triggered);
  weather_button = weather_button_init(on_weather_button_clicked);
  htop_button = htop_button_init(on_htop_button_clicked);
  language = language_init();
  cpu = cpu_init();
  memory = memory_init(on_memory_clicked);
  network = network_init(on_network_settings_clicked, on_network_ping_clicked,
                         on_nework_address_clicked);
  clock_ = clock_init();
  power = power_init(on_power_clicked);

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
