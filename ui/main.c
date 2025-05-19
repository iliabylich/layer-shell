#include "bindings.h"
#include "ui/include/builder.h"
#include "ui/include/css.h"
#include "ui/include/htop.h"
#include "ui/include/icons.h"
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
#include "ui/include/top_bar/sound.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/weather_button.h"
#include "ui/include/top_bar/workspaces.h"
#include "ui/include/weather.h"
#include <gtk/gtk.h>

GtkApplication *app;
IO_UiCtx *ui_ctx;
IO_IoCtx *io_ctx;
IO_IoThread *io_thread;

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
GtkWidget *sound;
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
  IO_CArray_Event events = io_poll_events(ui_ctx);
  bool keep_processing = true;
  for (size_t i = 0; i < events.len && keep_processing; i++) {
    IO_Event event = events.ptr[i];
    switch (event.tag) {
    case IO_Event_Workspaces: {
      workspaces_refresh(workspaces, event.workspaces.ids,
                         event.workspaces.active_id);
      break;
    }
    case IO_Event_ReloadStyles: {
      css_reload();
      break;
    }
    case IO_Event_Tray: {
      tray_refresh(tray, event.tray.apps);
      break;
    }
    case IO_Event_CurrentWeather: {
      weather_button_refresh(weather_button, event.current_weather.temperature,
                             event.current_weather.code);
      break;
    }
    case IO_Event_ForecastWeather: {
      weather_refresh(weather, event.forecast_weather);
      break;
    }
    case IO_Event_Language: {
      language_refresh(language, event.language.lang);
      break;
    }
    case IO_Event_Volume: {
      sound_refresh(sound, event.volume.volume, event.volume.muted);
      break;
    }
    case IO_Event_CpuUsage: {
      cpu_refresh(cpu, event.cpu_usage.usage_per_core);
      break;
    }
    case IO_Event_Memory: {
      memory_refresh(memory, event.memory.used, event.memory.total);
      break;
    }
    case IO_Event_WifiStatus: {
      network_refresh_wifi_status(network, event.wifi_status.wifi_status);
      break;
    }
    case IO_Event_NetworkSpeed: {
      network_refresh_network_speed(network, event.network_speed.upload_speed,
                                    event.network_speed.download_speed);
      break;
    }
    case IO_Event_NetworkList: {
      network_refresh_network_list(network, event.network_list.list);
      break;
    }
    case IO_Event_Time: {
      clock_refresh(clock_, event.time.time);
      break;
    }
    case IO_Event_ToggleSessionScreen: {
      session_toggle(session);
      break;
    }
    case IO_Event_Exit: {
      fprintf(stderr, "[UI] Received exit...\n");
      io_finalize(ui_ctx, io_thread);
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
  io_hyprland_go_to_workspace(ui_ctx, idx);
}

static void on_theme_change_clicked() { io_change_theme(ui_ctx); }

static void on_tray_triggered(const uint8_t *uuid) {
  io_trigger_tray(ui_ctx, uuid);
}

static void on_weather_button_clicked() { weather_toggle(weather); }

static void on_htop_button_clicked() { htop_toggle(htop); }

static void on_memory_clicked() { io_spawn_system_monitor(ui_ctx); }

static void on_network_settings_clicked() { io_spawn_network_editor(ui_ctx); }

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

static void on_lock_clicked() { io_lock(ui_ctx); }
static void on_reboot_clicked() { io_reboot(ui_ctx); }
static void on_shutdown_clicked() { io_shutdown(ui_ctx); }
static void on_logout_clicked() { io_logout(ui_ctx); }

static void on_app_activate() {
  init_icons();
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
  sound = sound_init();
  cpu = cpu_init();
  memory = memory_init(on_memory_clicked);
  network = network_init(on_network_settings_clicked, on_network_ping_clicked,
                         on_nework_address_clicked);
  clock_ = clock_init();
  power = power_init(on_power_clicked);

  gtk_window_present(GTK_WINDOW(top_bar));

  g_timeout_add(50, G_SOURCE_FUNC(poll_events), NULL);

  io_thread = io_spawn_thread(io_ctx);
}

int main(int argc, char **argv) {
  setenv("GSK_RENDERER", "cairo", true);
  IO_Ctx ctx = io_init();
  ui_ctx = ctx.ui;
  io_ctx = ctx.io;

  app = gtk_application_new("org.me.LayerShell", G_APPLICATION_DEFAULT_FLAGS);
  g_signal_connect(app, "activate", on_app_activate, NULL);
  g_signal_connect(app, "startup", css_load, NULL);
  g_application_run(G_APPLICATION(app), argc, argv);
  g_object_unref(app);

  return 0;
}
