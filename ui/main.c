#include "bindings.h"
#include "ui/caps_lock_window.h"
#include "ui/css.h"
#include "ui/io_model.h"
#include "ui/logger.h"
#include "ui/ping_window.h"
#include "ui/session_window.h"
#include "ui/sound_window.h"
#include "ui/terminal_window.h"
#include "ui/top_bar.h"
#include "ui/weather_window.h"
#include <glib-unix.h>
#include <gtk/gtk.h>

LOGGER("main.c", 0)

GtkApplication *app;

GtkWidget *top_bar;
GtkWidget *weather_window;
GtkWidget *terminal_window;
GtkWidget *ping_window;
GtkWidget *session_window;
GtkWidget *sound_window;
GtkWidget *caps_lock_window;

const IO_IOConfig *config;
IOModel *model;
bool exiting = false;

static void remove_window(GtkWidget **win) {
  gtk_application_remove_window(app, GTK_WINDOW(*win));
  g_clear_pointer(win, g_object_unref);
}

static void window_toggle(GtkWidget *, gpointer data) {
  GtkWidget *window = data;
  gtk_widget_set_visible(window, !gtk_widget_get_visible(window));
}

static void state_toggle(GtkWidget *, gpointer data) {
  WindowModel *state = NULL;
  const char *model_prop = data;
  g_object_get(model, model_prop, &state, NULL);
  if (state == NULL) {
    return;
  }
  gboolean visible = false;
  g_object_get(state, "visible", &visible, NULL);
  g_object_set(state, "visible", !visible, NULL);
  g_clear_object(&state);
}

static void event_received(const IO_Event *event) {
#define SET(prop, val) g_object_set(model, prop, val, NULL)

  switch (event->tag) {
  case IO_Event_Workspaces:
    io_model_set_workspaces(model, event->workspaces.workspaces);
    break;
  case IO_Event_ReloadStyles:
    css_reload();
    break;
  case IO_Event_TrayAppAdded:
    io_model_tray_add_app(model, event->tray_app_added.service,
                          event->tray_app_added.icon,
                          event->tray_app_added.items);
    break;
  case IO_Event_TrayAppRemoved:
    io_model_tray_remove_app(model, event->tray_app_removed.service);
    break;
  case IO_Event_TrayAppIconUpdated:
    io_model_tray_set_icon(model, event->tray_app_icon_updated.service,
                           event->tray_app_icon_updated.icon);
    break;
  case IO_Event_TrayAppMenuUpdated:
    io_model_tray_set_menu(model, event->tray_app_menu_updated.service,
                           event->tray_app_menu_updated.items);
    break;
  case IO_Event_Weather:
    io_model_set_weather(model, event->weather);
    break;
  case IO_Event_Language:
    SET("language-text", event->language.lang);
    break;
  case IO_Event_CpuUsage:
    io_model_set_cpu(model, event->cpu_usage.usage_per_core);
    break;
  case IO_Event_Memory:
    SET("memory-used", event->memory.used);
    SET("memory-total", event->memory.total);
    break;
  case IO_Event_NetworkSsid:
    SET("network-ssid", event->network_ssid.ssid);
    break;
  case IO_Event_NetworkStrength:
    SET("network-strength", event->network_strength.strength);
    break;
  case IO_Event_DownloadSpeed:
    SET("download-bytes-per-sec", event->download_speed.bytes_per_sec);
    break;
  case IO_Event_UploadSpeed:
    SET("upload-bytes-per-sec", event->upload_speed.bytes_per_sec);
    break;
  case IO_Event_Clock:
    SET("clock-unix-seconds", event->clock.unix_seconds);
    break;
  case IO_Event_ToggleSessionScreen:
    state_toggle(NULL, "session-window-model");
    break;
  case IO_Event_InitialSound:
    SET("sound-volume", event->initial_sound.volume);
    SET("sound-muted", event->initial_sound.muted);
    SET("sound-ready", true);
    break;
  case IO_Event_VolumeChanged:
    SET("sound-volume", event->volume_changed.volume);
    break;
  case IO_Event_MuteChanged:
    SET("sound-muted", event->mute_changed.muted);
    break;
  case IO_Event_CapsLockToggled:
    SET("caps-lock-enabled", event->caps_lock_toggled.enabled);
    break;
  case IO_Event_Exit:
    LOG("Received exit...");
    io_deinit();
    LOG("Removing windows...");
    remove_window(&top_bar);
    remove_window(&weather_window);
    remove_window(&terminal_window);
    remove_window(&ping_window);
    remove_window(&session_window);
    g_application_quit(G_APPLICATION(app));
    LOG("Quit done.");
    exiting = true;
    break;
  }

#undef SET
}

static gboolean read_io_events(gint, GIOCondition, gpointer) {
  io_handle_readable();

  return exiting ? G_SOURCE_REMOVE : G_SOURCE_CONTINUE;
}

static void workspace_switched(TopBar *, guint num) {
  io_hyprland_go_to_workspace(num);
}
static void tray_triggered(TopBar *, const char *uuid) {
  io_trigger_tray(uuid);
}

static void create_widgets() {
  SoundWindowModel *sound_window_model = NULL;
  WeatherWindowModel *weather_window_model = NULL;
  SessionWindowModel *session_window_model = NULL;
  model = io_model_new();
  g_object_get(model, "sound-window-model", &sound_window_model, NULL);
  g_object_get(model, "weather-window-model", &weather_window_model,
               "session-window-model", &session_window_model, NULL);
  top_bar = top_bar_new(app, model);
  top_bar_set_terminal_label(TOP_BAR(top_bar), config->terminal.label);

#define CONNECT(obj, signal, callback, data)                                   \
  g_signal_connect(obj, signal, G_CALLBACK(callback), data)

  CONNECT(top_bar, "workspace-switched", workspace_switched, NULL);
  CONNECT(top_bar, "change-theme-clicked", io_change_theme, NULL);
  CONNECT(top_bar, "tray-triggered", tray_triggered, NULL);
  CONNECT(top_bar, "memory-clicked", io_spawn_system_monitor, NULL);
  CONNECT(top_bar, "network-settings-clicked", io_spawn_wifi_editor, NULL);
  CONNECT(top_bar, "bluetooth-clicked", io_spawn_bluetooh_editor, NULL);

  weather_window = weather_window_new(app, model, weather_window_model);
  terminal_window = terminal_window_new(app);
  ping_window = ping_window_new(app);

  session_window = session_window_new(app, session_window_model);
  CONNECT(session_window, "clicked-lock", io_lock, NULL);
  CONNECT(session_window, "clicked-shutdown", io_shutdown, NULL);
  CONNECT(session_window, "clicked-reboot", io_reboot, NULL);
  CONNECT(session_window, "clicked-logout", io_logout, NULL);

  CONNECT(top_bar, "weather-clicked", state_toggle, "weather-window-model");
  CONNECT(top_bar, "terminal-clicked", window_toggle, terminal_window);
  CONNECT(top_bar, "network-ping-clicked", window_toggle, ping_window);
  CONNECT(top_bar, "power-clicked", state_toggle, "session-window-model");

#undef CONNECT

  sound_window = sound_window_new(app, model, sound_window_model);
  caps_lock_window = caps_lock_window_new(app, model);
  g_clear_object(&sound_window_model);
  g_clear_object(&weather_window_model);
  g_clear_object(&session_window_model);

  g_unix_fd_add(io_as_raw_fd(), G_IO_IN, read_io_events, NULL);
  gtk_window_present(GTK_WINDOW(top_bar));
}

int main(int argc, char **argv) {
#define CONNECT(obj, signal, callback, data)                                   \
  g_signal_connect(obj, signal, G_CALLBACK(callback), data)

  setenv("GSK_RENDERER", "cairo", true);
  io_init(event_received, true);
  config = io_get_config();

  app = gtk_application_new("org.me.LayerShell", G_APPLICATION_DEFAULT_FLAGS);
  CONNECT(app, "activate", create_widgets, NULL);
  CONNECT(app, "startup", css_load, NULL);
  g_application_run(G_APPLICATION(app), argc, argv);
  g_object_unref(app);

#undef CONNECT

  return 0;
}
