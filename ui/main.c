#include "bindings.h"
#include "ui/caps_lock_overlay.h"
#include "ui/css.h"
#include "ui/gobject_helper.h"
#include "ui/logger.h"
#include "ui/ping_overlay.h"
#include "ui/session_overlay.h"
#include "ui/sound_overlay.h"
#include "ui/terminal_overlay.h"
#include "ui/top_bar.h"
#include "ui/view_models/io_model.h"
#include "ui/weather_overlay.h"
#include <glib-unix.h>
#include <gtk/gtk.h>

LOGGER("main.c", 0)

GtkApplication *app;

GtkWidget *top_bar;
GtkWidget *weather_overlay;
GtkWidget *terminal_overlay;
GtkWidget *ping_overlay;
GtkWidget *session_overlay;
GtkWidget *sound_overlay;
GtkWidget *caps_lock_overlay;

const IO_IOConfig *config;
IOModel *model;
bool exiting = false;

static void remove_window(GtkWidget **win) {
  gtk_application_remove_window(app, GTK_WINDOW(*win));
  g_clear_pointer(win, g_object_unref);
}

static void toggle_overlay_by_name(GtkWidget *, gpointer data) {
  const char *overlay_prop = data;
  gobject_toggle_nested(G_OBJECT(model), "overlays", overlay_prop);
}

static void event_received(const IO_Event *event) {
#define SET(child, prop, val)                                                  \
  gobject_set_nested(G_OBJECT(model), child, prop, val)

  switch (event->tag) {
  case IO_Event_Workspaces:
    SET("workspaces", "data", (gpointer)&event->workspaces.workspaces);
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
    SET("weather", "data", (gpointer)&event->weather);
    break;
  case IO_Event_Language:
    SET("language", "text", event->language.lang);
    break;
  case IO_Event_CpuUsage:
    SET("cpu", "data", (gpointer)&event->cpu_usage.usage_per_core);
    break;
  case IO_Event_Memory:
    SET("memory", "used", event->memory.used);
    SET("memory", "total", event->memory.total);
    break;
  case IO_Event_NetworkSsid:
    SET("network", "ssid", event->network_ssid.ssid);
    break;
  case IO_Event_NetworkStrength:
    SET("network", "strength", event->network_strength.strength);
    break;
  case IO_Event_DownloadSpeed:
    SET("network", "download-bytes-per-sec",
        event->download_speed.bytes_per_sec);
    break;
  case IO_Event_UploadSpeed:
    SET("network", "upload-bytes-per-sec", event->upload_speed.bytes_per_sec);
    break;
  case IO_Event_Clock: {
    SET("clock", "unix-seconds", event->clock.unix_seconds);
    break;
  }
  case IO_Event_ToggleSessionScreen:
    toggle_overlay_by_name(NULL, "session");
    break;
  case IO_Event_InitialSound:
    SET("sound", "initial", (gpointer)&event->initial_sound);
    break;
  case IO_Event_VolumeChanged:
    SET("sound", "volume", event->volume_changed.volume);
    break;
  case IO_Event_MuteChanged:
    SET("sound", "muted", event->mute_changed.muted);
    break;
  case IO_Event_CapsLockToggled:
    SET("caps-lock", "enabled", event->caps_lock_toggled.enabled);
    break;
  case IO_Event_Exit:
    LOG("Received exit...");
    io_deinit();
    LOG("Removing windows...");
    remove_window(&top_bar);
    remove_window(&weather_overlay);
    remove_window(&terminal_overlay);
    remove_window(&ping_overlay);
    remove_window(&session_overlay);
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
  model = io_model_new();
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

  weather_overlay = weather_overlay_new(app, model);
  terminal_overlay = terminal_overlay_new(app, model);
  ping_overlay = ping_overlay_new(app, model);

  session_overlay = session_overlay_new(app, model);
  CONNECT(session_overlay, "clicked-lock", io_lock, NULL);
  CONNECT(session_overlay, "clicked-shutdown", io_shutdown, NULL);
  CONNECT(session_overlay, "clicked-reboot", io_reboot, NULL);
  CONNECT(session_overlay, "clicked-logout", io_logout, NULL);

  CONNECT(top_bar, "weather-clicked", toggle_overlay_by_name, "weather");
  CONNECT(top_bar, "terminal-clicked", toggle_overlay_by_name, "terminal");
  CONNECT(top_bar, "network-ping-clicked", toggle_overlay_by_name, "ping");
  CONNECT(top_bar, "power-clicked", toggle_overlay_by_name, "session");

#undef CONNECT

  sound_overlay = sound_overlay_new(app, model);
  caps_lock_overlay = caps_lock_overlay_new(app, model);

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
