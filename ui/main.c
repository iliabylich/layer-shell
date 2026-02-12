#include "bindings.h"
#include "ui/io_model.h"
#include "ui/caps_lock_window.h"
#include "ui/css.h"
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

void *io;
const IO_IOConfig *config;
IOModel *model;
bool exiting = false;

static void remove_window(GtkWidget **win) {
  gtk_application_remove_window(app, GTK_WINDOW(*win));
  g_clear_pointer(win, g_object_unref);
}

static void on_event(const IO_Event *event) {
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
    io_model_set_weather(model, event->weather.temperature,
                            event->weather.code);
    weather_window_refresh_hourly_forecast(WEATHER_WINDOW(weather_window),
                                           event->weather.hourly_forecast);
    weather_window_refresh_daily_forecast(WEATHER_WINDOW(weather_window),
                                          event->weather.daily_forecast);
    break;
  case IO_Event_Language:
    io_model_set_language(model, event->language.lang);
    break;
  case IO_Event_CpuUsage:
    io_model_set_cpu(model, event->cpu_usage.usage_per_core);
    break;
  case IO_Event_Memory:
    io_model_set_memory(model, event->memory.used, event->memory.total);
    break;
  case IO_Event_NetworkSsid:
    io_model_set_network_ssid(model, event->network_ssid.ssid);
    break;
  case IO_Event_NetworkStrength:
    io_model_set_network_strength(model, event->network_strength.strength);
    break;
  case IO_Event_DownloadSpeed:
    io_model_set_download_speed(model, event->download_speed.speed);
    break;
  case IO_Event_UploadSpeed:
    io_model_set_upload_speed(model, event->upload_speed.speed);
    break;
  case IO_Event_Clock:
    io_model_set_clock_text(model, event->clock.time);
    break;
  case IO_Event_ToggleSessionScreen:
    session_window_toggle(SESSION_WINDOW(session_window));
    break;
  case IO_Event_InitialSound:
    sound_window_set_initial_sound(SOUND_WINDOW(sound_window),
                                   event->initial_sound.volume,
                                   event->initial_sound.muted);
    break;
  case IO_Event_VolumeChanged:
    sound_window_refresh_volume(SOUND_WINDOW(sound_window),
                                event->volume_changed.volume);
    break;
  case IO_Event_MuteChanged:
    sound_window_refresh_mute(SOUND_WINDOW(sound_window),
                              event->mute_changed.muted);
    break;
  case IO_Event_CapsLockToggled:
    caps_lock_window_refresh(CAPS_LOCK_WINDOW(caps_lock_window),
                             event->caps_lock_toggled.enabled);
    break;
  case IO_Event_Exit:
    LOG("Received exit...");
    io_deinit(io);
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
}

static gboolean on_new_events(gint, GIOCondition, gpointer);

static void on_workspace_switched(TopBar *, guint num) {
  io_hyprland_go_to_workspace(io, num);
}
static void on_change_theme_clicked() { io_change_theme(io); }
static void on_tray_triggered(TopBar *, const char *uuid) {
  io_trigger_tray(io, uuid);
}
static void on_weather_clicked() {
  weather_window_toggle(WEATHER_WINDOW(weather_window));
}
static void on_terminal_clicked() {
  terminal_window_toggle(TERMINAL_WINDOW(terminal_window));
}
static void on_memory_clicked() { io_spawn_system_monitor(io); }
static void on_network_settings_clicked() { io_spawn_wifi_editor(io); }
static void on_network_ping_clicked() {
  ping_window_toggle(PING_WINDOW(ping_window));
}
static void on_bluetooth_clicked() { io_spawn_bluetooh_editor(io); }
static void on_power_clicked() {
  session_window_toggle(SESSION_WINDOW(session_window));
}
static void on_lock_clicked() { io_lock(io); }
static void on_reboot_clicked() { io_reboot(io); }
static void on_shutdown_clicked() { io_shutdown(io); }
static void on_logout_clicked() { io_logout(io); }

static void on_app_activate() {
  model = io_model_new();
  top_bar = top_bar_new(app);
  top_bar_set_model(TOP_BAR(top_bar), model);
  top_bar_set_terminal_label(TOP_BAR(top_bar), config->terminal.label);

#define CONNECT(signal, callback)                                              \
  g_signal_connect(top_bar, signal, G_CALLBACK(callback), NULL)

  CONNECT("workspace-switched", on_workspace_switched);
  CONNECT("change-theme-clicked", on_change_theme_clicked);
  CONNECT("tray-triggered", on_tray_triggered);
  CONNECT("weather-clicked", on_weather_clicked);
  CONNECT("terminal-clicked", on_terminal_clicked);
  CONNECT("memory-clicked", on_memory_clicked);
  CONNECT("network-settings-clicked", on_network_settings_clicked);
  CONNECT("network-ping-clicked", on_network_ping_clicked);
  CONNECT("bluetooth-clicked", on_bluetooth_clicked);
  CONNECT("power-clicked", on_power_clicked);

#undef CONNECT

  weather_window = weather_window_new(app);
  terminal_window = terminal_window_new(app);
  ping_window = ping_window_new(app);

  session_window = session_window_new(app);
  g_signal_connect(session_window, "clicked-lock", G_CALLBACK(on_lock_clicked),
                   NULL);
  g_signal_connect(session_window, "clicked-shutdown",
                   G_CALLBACK(on_shutdown_clicked), NULL);
  g_signal_connect(session_window, "clicked-reboot",
                   G_CALLBACK(on_reboot_clicked), NULL);
  g_signal_connect(session_window, "clicked-logout",
                   G_CALLBACK(on_logout_clicked), NULL);

  sound_window = sound_window_new(app);
  caps_lock_window = caps_lock_window_new(app);

  g_unix_fd_add(io_as_raw_fd(), G_IO_IN, on_new_events, NULL);
  gtk_window_present(GTK_WINDOW(top_bar));
}

static gboolean on_new_events(gint, GIOCondition, gpointer) {
  io_handle_readable(io);

  return exiting ? G_SOURCE_REMOVE : G_SOURCE_CONTINUE;
}

int main(int argc, char **argv) {
  setenv("GSK_RENDERER", "cairo", true);
  io = io_init(on_event);
  config = io_get_config(io);

  app = gtk_application_new("org.me.LayerShell", G_APPLICATION_DEFAULT_FLAGS);
  g_signal_connect(app, "activate", on_app_activate, NULL);
  g_signal_connect(app, "startup", css_load, NULL);
  g_application_run(G_APPLICATION(app), argc, argv);
  g_object_unref(app);

  return 0;
}
