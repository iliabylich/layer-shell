#include "bindings.h"
#include "glib-object.h"
#include "ui/include/css.h"
#include "ui/include/htop.h"
#include "ui/include/icons.h"
#include "ui/include/launcher.h"
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
GtkWidget *launcher;

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
      workspaces_refresh(WORKSPACES(workspaces), event.workspaces.ids,
                         event.workspaces.active_id);
      break;
    }
    case IO_Event_ReloadStyles: {
      css_reload();
      break;
    }
    case IO_Event_Tray: {
      tray_refresh(TRAY(tray), event.tray.apps);
      break;
    }
    case IO_Event_CurrentWeather: {
      weather_button_refresh(WEATHER_BUTTON(weather_button),
                             event.current_weather.temperature,
                             event.current_weather.code);
      break;
    }
    case IO_Event_ForecastWeather: {
      weather_refresh(WEATHER(weather), event.forecast_weather);
      break;
    }
    case IO_Event_Language: {
      language_refresh(LANGUAGE(language), event.language.lang);
      break;
    }
    case IO_Event_Volume: {
      sound_refresh(SOUND(sound), event.volume.volume, event.volume.muted);
      break;
    }
    case IO_Event_CpuUsage: {
      cpu_refresh(CPU(cpu), event.cpu_usage.usage_per_core);
      break;
    }
    case IO_Event_Memory: {
      memory_refresh(MEMORY(memory), event.memory.used, event.memory.total);
      break;
    }
    case IO_Event_WifiStatus: {
      network_refresh_wifi_status(NETWORK(network),
                                  event.wifi_status.wifi_status);
      break;
    }
    case IO_Event_NetworkSpeed: {
      network_refresh_network_speed(NETWORK(network),
                                    event.network_speed.upload_speed,
                                    event.network_speed.download_speed);
      break;
    }
    case IO_Event_NetworkList: {
      network_refresh_network_list(NETWORK(network), event.network_list.list);
      break;
    }
    case IO_Event_Time: {
      clock_refresh(CLOCK(clock_), event.time.time);
      break;
    }
    case IO_Event_ToggleSessionScreen: {
      window_toggle(GTK_WINDOW(session));
      break;
    }
    case IO_Event_Launcher: {
      launcher_refresn(LAUNCHER(launcher), event.launcher.apps);
      break;
    }
    case IO_Event_ToggleLauncher: {
      launcher_toggle_and_reset(LAUNCHER(launcher));
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
      drop_window(GTK_WINDOW(launcher));
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

static void switch_workspace(Workspaces *, size_t idx) {
  io_hyprland_go_to_workspace(ui_ctx, idx);
}

static void switch_theme() { io_change_theme(ui_ctx); }

static void trigger_tray(Tray *, const uint8_t *uuid) {
  io_trigger_tray(ui_ctx, uuid);
}

static void toggle_weather() { window_toggle(GTK_WINDOW(weather)); }

static void toggle_htop() { window_toggle(GTK_WINDOW(htop)); }

static void open_system_monitor() { io_spawn_system_monitor(ui_ctx); }

static void spawn_network_editor() { io_spawn_network_editor(ui_ctx); }

static void toggle_ping() { window_toggle(GTK_WINDOW(ping)); }

static void copy_ip_to_clipboard(Network *, char *ip) {
  GdkDisplay *display = gdk_display_get_default();
  GdkClipboard *clipboard = gdk_display_get_clipboard(display);
  gdk_clipboard_set_text(clipboard, ip);

  char format[100];
  sprintf(format, "Copied %s", ip);
  GNotification *notification = g_notification_new(format);
  g_application_send_notification(G_APPLICATION(app), NULL, notification);
}

static void toggle_session_window() { window_toggle(GTK_WINDOW(session)); }

static void lock() { io_lock(ui_ctx); }
static void reboot() { io_reboot(ui_ctx); }
static void shutdown() { io_shutdown(ui_ctx); }
static void logout() { io_logout(ui_ctx); }

static void launcher_exec_selected() { io_launcher_exec_selected(ui_ctx); }
static void launcher_go_up() { io_launcher_go_up(ui_ctx); }
static void launcher_go_down() { io_launcher_go_down(ui_ctx); }
static void launcher_reset() { io_launcher_reset(ui_ctx); }
static void launcher_set_search(Launcher *, const uint8_t *search) {
  io_launcher_set_search(ui_ctx, search);
}

static void on_app_activate() {
  init_icons();

  top_bar = top_bar_new(app);
  weather = weather_new(app);
  htop = htop_new(app);
  ping = ping_new(app);
  session = session_new(app);
  launcher = launcher_new(app);

  workspaces = workspaces_new();
  change_theme = change_theme_new();

  tray = tray_new();
  weather_button = weather_button_new();
  htop_button = htop_button_new();
  language = language_new();
  sound = sound_new();
  cpu = cpu_new();
  memory = memory_new();
  network = network_new();
  clock_ = clock_new();
  power = power_new();

  top_bar_push_left(TOP_BAR(top_bar), workspaces);
  top_bar_push_left(TOP_BAR(top_bar), change_theme);

  top_bar_push_right(TOP_BAR(top_bar), tray);
  top_bar_push_right(TOP_BAR(top_bar), weather_button);
  top_bar_push_right(TOP_BAR(top_bar), htop_button);
  top_bar_push_right(TOP_BAR(top_bar), language);
  top_bar_push_right(TOP_BAR(top_bar), sound);
  top_bar_push_right(TOP_BAR(top_bar), cpu);
  top_bar_push_right(TOP_BAR(top_bar), memory);
  top_bar_push_right(TOP_BAR(top_bar), network);
  top_bar_push_right(TOP_BAR(top_bar), clock_);
  top_bar_push_right(TOP_BAR(top_bar), power);

#define CONNECT(widget, signal, callback)                                      \
  g_signal_connect(widget, signal, G_CALLBACK(callback), NULL);

  CONNECT(workspaces, "switched", switch_workspace);
  CONNECT(change_theme, "clicked", switch_theme);
  CONNECT(tray, "triggered", trigger_tray);
  CONNECT(weather_button, "clicked", toggle_weather);
  CONNECT(htop_button, "clicked", toggle_htop);
  CONNECT(memory, "clicked", open_system_monitor);
  CONNECT(network, "settings-clicked", spawn_network_editor);
  CONNECT(network, "ping-clicked", toggle_ping);
  CONNECT(network, "network-clicked", copy_ip_to_clipboard);
  CONNECT(power, "clicked", toggle_session_window);
  CONNECT(session, "lock", lock);
  CONNECT(session, "reboot", reboot);
  CONNECT(session, "shutdown", shutdown);
  CONNECT(session, "logout", logout);
  CONNECT(launcher, "exec-selected", launcher_exec_selected);
  CONNECT(launcher, "go-up", launcher_go_up);
  CONNECT(launcher, "go-down", launcher_go_down);
  CONNECT(launcher, "reset", launcher_reset);
  CONNECT(launcher, "set-search", launcher_set_search);
#undef CONNECT

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
