#ifndef WIDGETS_H
#define WIDGETS_H

#include "bindings.h"
#include "icons.h"
#include "pango/pango-layout.h"
#include "weather.h"
#include <gdk/gdk.h>
#include <glib-object.h>
#include <glib.h>
#include <glibconfig.h>
#include <gtk/gtk.h>
#include <gtk/gtkshortcut.h>
#include <gtk4-layer-shell.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <vte/vte.h>

GtkWindow *top_bar_window;

GtkBox *workspaces_widget;
GtkButton *workspace_buttons[10];

GtkButton *htop_widget;

GtkButton *weather_widget;
GtkLabel *weather_label;

GtkCenterBox *language_widget;
GtkLabel *language_label;

GtkBox *sound_widget;
GtkImage *sound_image;
GtkScale *sound_scale;

GtkBox *cpu_widget;
GtkLabel *cpu_labels[12];
const char *CPU_INDICATORS[] = {
    "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
};
const size_t CPU_INDICATORS_COUNT = sizeof(CPU_INDICATORS) / sizeof(char *);

GtkButton *ram_widget;
GtkLabel *ram_label;

GtkButton *network_widget;
GtkLabel *network_label;
GtkImage *network_image;

GtkCenterBox *time_widget;
GtkLabel *time_label;

GtkButton *session_widget;

void init_top_bar(void) {
  top_bar_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(top_bar_window), "TopBarWindow");

  GtkCenterBox *layout = GTK_CENTER_BOX(gtk_center_box_new());
  gtk_widget_add_css_class(GTK_WIDGET(layout), "main-wrapper");
  gtk_window_set_child(top_bar_window, GTK_WIDGET(layout));

  GtkBox *left = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8));
  gtk_center_box_set_start_widget(layout, GTK_WIDGET(left));

  GtkBox *right = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4));
  gtk_center_box_set_end_widget(layout, GTK_WIDGET(right));

  // workspaces
  workspaces_widget = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(workspaces_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(workspaces_widget), "workspaces");
  for (size_t i = 0; i < 10; i++) {
    GtkButton *button = GTK_BUTTON(gtk_button_new());
    char buffer[3];
    sprintf(buffer, "%lu", i + 1);
    GtkLabel *label = GTK_LABEL(gtk_label_new(buffer));
    gtk_button_set_child(button, GTK_WIDGET(label));
    gtk_box_append(workspaces_widget, GTK_WIDGET(button));
    workspace_buttons[i] = button;
  }
  gtk_box_append(left, GTK_WIDGET(workspaces_widget));

  // htop
  htop_widget = GTK_BUTTON(gtk_button_new());
  gtk_widget_add_css_class(GTK_WIDGET(htop_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(htop_widget), "terminal");
  gtk_widget_add_css_class(GTK_WIDGET(htop_widget), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(htop_widget), "clickable");
  GtkLabel *htop_label = GTK_LABEL(gtk_label_new("Htop"));
  gtk_button_set_child(htop_widget, GTK_WIDGET(htop_label));
  gtk_box_append(right, GTK_WIDGET(htop_widget));

  // weather
  weather_label = GTK_LABEL(gtk_label_new("--"));
  weather_widget = GTK_BUTTON(gtk_button_new());
  gtk_widget_add_css_class(GTK_WIDGET(weather_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(weather_widget), "weather");
  gtk_widget_add_css_class(GTK_WIDGET(weather_widget), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(weather_widget), "clickable");
  gtk_button_set_child(weather_widget, GTK_WIDGET(weather_label));
  gtk_box_append(right, GTK_WIDGET(weather_widget));

  // language
  language_label = GTK_LABEL(gtk_label_new("--"));
  language_widget = GTK_CENTER_BOX(gtk_center_box_new());
  gtk_widget_add_css_class(GTK_WIDGET(language_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(language_widget), "language");
  gtk_widget_add_css_class(GTK_WIDGET(language_widget), "padded");
  gtk_center_box_set_center_widget(language_widget, GTK_WIDGET(language_label));
  gtk_box_append(right, GTK_WIDGET(language_widget));

  // sound
  sound_widget = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(sound_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(sound_widget), "sound");
  gtk_widget_add_css_class(GTK_WIDGET(sound_widget), "padded");
  sound_image = GTK_IMAGE(gtk_image_new());
  gtk_image_set_from_icon_name(sound_image, "dialog-question");
  gtk_box_append(sound_widget, GTK_WIDGET(sound_image));
  sound_scale = GTK_SCALE(
      gtk_scale_new(GTK_ORIENTATION_HORIZONTAL,
                    gtk_adjustment_new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0)));
  gtk_widget_add_css_class(GTK_WIDGET(sound_scale), "sound-slider");
  gtk_box_append(sound_widget, GTK_WIDGET(sound_scale));
  gtk_box_append(right, GTK_WIDGET(sound_widget));

  // cpu
  cpu_widget = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 3));
  gtk_widget_add_css_class(GTK_WIDGET(cpu_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(cpu_widget), "cpu");
  gtk_widget_add_css_class(GTK_WIDGET(cpu_widget), "padded");
  for (size_t i = 0; i < 12; i++) {
    GtkLabel *label = GTK_LABEL(gtk_label_new(NULL));
    gtk_label_set_use_markup(label, true);
    gtk_box_append(cpu_widget, GTK_WIDGET(label));
    cpu_labels[i] = label;
  }
  gtk_box_append(right, GTK_WIDGET(cpu_widget));

  // ram
  ram_label = GTK_LABEL(gtk_label_new(NULL));
  ram_widget = GTK_BUTTON(gtk_button_new());
  gtk_widget_add_css_class(GTK_WIDGET(ram_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(ram_widget), "memory");
  gtk_widget_add_css_class(GTK_WIDGET(ram_widget), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(ram_widget), "clickable");
  gtk_button_set_child(ram_widget, GTK_WIDGET(ram_label));
  gtk_box_append(right, GTK_WIDGET(ram_widget));

  // network
  network_label = GTK_LABEL(gtk_label_new("--"));
  network_image = GTK_IMAGE(gtk_image_new());
  gtk_image_set_from_gicon(network_image, WIFI_ICON);
  GtkBox *network_wrapper = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  gtk_box_append(network_wrapper, GTK_WIDGET(network_label));
  gtk_box_append(network_wrapper, GTK_WIDGET(network_image));
  network_widget = GTK_BUTTON(gtk_button_new());
  gtk_widget_add_css_class(GTK_WIDGET(network_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(network_widget), "network");
  gtk_widget_add_css_class(GTK_WIDGET(network_widget), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(network_widget), "clickable");
  gtk_widget_set_cursor(GTK_WIDGET(network_widget),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_button_set_child(network_widget, GTK_WIDGET(network_wrapper));
  gtk_box_append(right, GTK_WIDGET(network_widget));

  // clock
  time_label = GTK_LABEL(gtk_label_new("--"));
  time_widget = GTK_CENTER_BOX(gtk_center_box_new());
  gtk_widget_add_css_class(GTK_WIDGET(time_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(time_widget), "clock");
  gtk_widget_add_css_class(GTK_WIDGET(time_widget), "padded");
  gtk_center_box_set_center_widget(time_widget, GTK_WIDGET(time_label));
  gtk_box_append(right, GTK_WIDGET(time_widget));

  // session
  session_widget = GTK_BUTTON(gtk_button_new());
  gtk_widget_add_css_class(GTK_WIDGET(session_widget), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(session_widget), "power");
  gtk_widget_add_css_class(GTK_WIDGET(session_widget), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(session_widget), "clickable");
  gtk_widget_set_cursor(GTK_WIDGET(network_widget),
                        gdk_cursor_new_from_name("pointer", NULL));
  GtkImage *session_image = GTK_IMAGE(gtk_image_new());
  gtk_image_set_from_gicon(session_image, POWER_ICON);
  gtk_button_set_child(session_widget, GTK_WIDGET(session_image));
  gtk_box_append(right, GTK_WIDGET(session_widget));
}

GtkWindow *session_window;
GtkButton *lock_button;
GtkButton *reboot_button;
GtkButton *shutdown_button;
GtkButton *logout_button;

GtkButton *session_button_new(const char *text) {
  GtkButton *btn = GTK_BUTTON(gtk_button_new());
  gtk_widget_add_css_class(GTK_WIDGET(btn), "session-window-button");
  GtkLabel *label = GTK_LABEL(gtk_label_new(text));
  gtk_button_set_child(btn, GTK_WIDGET(label));
  return btn;
}

void init_session_screen(void) {
  session_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(session_window), "SessionWindow");
  gtk_widget_add_css_class(GTK_WIDGET(session_window), "session-window");

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  gtk_box_set_homogeneous(layout, true);
  gtk_box_set_spacing(layout, 200);
  gtk_widget_add_css_class(GTK_WIDGET(layout), "session-window-wrapper");
  gtk_window_set_child(session_window, GTK_WIDGET(layout));

  lock_button = session_button_new("Lock");
  gtk_box_append(layout, GTK_WIDGET(lock_button));

  reboot_button = session_button_new("Reboot");
  gtk_box_append(layout, GTK_WIDGET(reboot_button));

  shutdown_button = session_button_new("Shutdown");
  gtk_box_append(layout, GTK_WIDGET(shutdown_button));

  logout_button = session_button_new("Logout");
  gtk_box_append(layout, GTK_WIDGET(logout_button));
}

GtkWindow *launcher_window;
GtkSearchEntry *launcher_input;
typedef struct {
  GtkBox *wrapper;
  GtkImage *icon;
  GtkLabel *label;
} launcher_row_t;
launcher_row_t launcher_rows[5];

void init_launcher(void) {
  launcher_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(launcher_window), "LauncherWindow");
  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, 700);
  g_object_set_property(G_OBJECT(launcher_window), "width-request",
                        &width_request);

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(layout), "widget-launcher-wrapper");
  gtk_window_set_child(launcher_window, GTK_WIDGET(layout));

  launcher_input = GTK_SEARCH_ENTRY(gtk_search_entry_new());
  gtk_widget_add_css_class(GTK_WIDGET(launcher_input),
                           "widget-launcher-search-box");
  gtk_widget_set_hexpand(GTK_WIDGET(launcher_input), true);
  gtk_box_append(layout, GTK_WIDGET(launcher_input));

  GtkScrolledWindow *scroll = GTK_SCROLLED_WINDOW(gtk_scrolled_window_new());
  gtk_widget_add_css_class(GTK_WIDGET(scroll), "widget-launcher-scroll-list");
  gtk_widget_set_can_focus(GTK_WIDGET(scroll), false);
  gtk_box_append(layout, GTK_WIDGET(scroll));

  GtkBox *content = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_scrolled_window_set_child(scroll, GTK_WIDGET(content));

  for (size_t i = 0; i < 5; i++) {
    GtkBox *row = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
    gtk_widget_add_css_class(GTK_WIDGET(row), "widget-launcher-row");

    GtkImage *image = GTK_IMAGE(gtk_image_new());
    gtk_image_set_icon_size(image, GTK_ICON_SIZE_LARGE);

    GtkLabel *label = GTK_LABEL(gtk_label_new("..."));
    gtk_label_set_xalign(label, 0.0);
    gtk_widget_set_valign(GTK_WIDGET(label), GTK_ALIGN_CENTER);
    gtk_label_set_ellipsize(label, PANGO_ELLIPSIZE_END);

    gtk_box_append(row, GTK_WIDGET(image));
    gtk_box_append(row, GTK_WIDGET(label));

    gtk_box_append(content, GTK_WIDGET(row));

    launcher_rows[i] =
        (launcher_row_t){.wrapper = row, .icon = image, .label = label};
  }
}

GtkWindow *networks_window;
typedef struct {
  GtkCenterBox *wrapper;
  GtkLabel *label;
} network_row_t;
network_row_t networks_rows[5];
network_row_t network_settings_row;
network_row_t network_exit_row;

network_row_t network_row_new(const char *text, const char *icon_name) {
  GtkCenterBox *row = GTK_CENTER_BOX(gtk_center_box_new());
  gtk_widget_add_css_class(GTK_WIDGET(row), "widget-network-row");
  gtk_orientable_set_orientation(GTK_ORIENTABLE(row),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_widget_set_halign(GTK_WIDGET(row), GTK_ALIGN_FILL);

  GtkLabel *label = GTK_LABEL(gtk_label_new(text));
  gtk_label_set_justify(label, GTK_JUSTIFY_LEFT);
  gtk_label_set_xalign(label, 0.0);
  gtk_center_box_set_start_widget(row, GTK_WIDGET(label));

  GtkImage *image = GTK_IMAGE(gtk_image_new());
  gtk_image_set_from_icon_name(image, icon_name);
  gtk_image_set_icon_size(image, GTK_ICON_SIZE_LARGE);
  gtk_image_set_pixel_size(image, 30);
  gtk_center_box_set_end_widget(row, GTK_WIDGET(image));

  return (network_row_t){.wrapper = row, .label = label};
}

void init_networks(void) {
  networks_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(networks_window), "NetworksWindow");
  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, 700);
  g_object_set_property(G_OBJECT(networks_window), "width-request",
                        &width_request);

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(layout), "widget-network-row-list");
  gtk_window_set_child(networks_window, GTK_WIDGET(layout));

  for (size_t i = 0; i < 5; i++) {
    network_row_t row = network_row_new("--", "edit-copy");
    gtk_box_append(layout, GTK_WIDGET(row.wrapper));
    networks_rows[i] = row;
  }

  network_settings_row =
      network_row_new("Settings (nmtui)", "preferences-system-network");
  gtk_box_append(layout, GTK_WIDGET(network_settings_row.wrapper));

  network_exit_row = network_row_new("Close", "window-close");
  gtk_box_append(layout, GTK_WIDGET(network_exit_row.wrapper));
}

GtkWindow *htop_window;

void init_htop(void) {
  htop_window = GTK_WINDOW(gtk_window_new());

  gtk_widget_set_name(GTK_WIDGET(htop_window), "HtopWindow");
  gtk_widget_add_css_class(GTK_WIDGET(htop_window), "widget-htop");

  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, 1000);
  g_object_set_property(G_OBJECT(htop_window), "width-request", &width_request);

  GValue height_request = G_VALUE_INIT;
  g_value_init(&height_request, G_TYPE_INT);
  g_value_set_int(&height_request, 700);
  g_object_set_property(G_OBJECT(htop_window), "height-request",
                        &height_request);
}

GtkWindow *weather_window;
typedef struct {
  GtkBox *wrapper;
  GtkLabel *label;
  GtkImage *image;
} weather_row_t;
#define WEATHER_HOURLY_ROWS_COUNT 10
weather_row_t weather_hourly_rows[WEATHER_HOURLY_ROWS_COUNT];
#define WEATHER_DAILY_ROWS_COUNT 6
weather_row_t weather_daily_rows[WEATHER_DAILY_ROWS_COUNT];

weather_row_t weather_row_new() {
  GtkBox *row = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  GtkLabel *label = GTK_LABEL(gtk_label_new("..."));
  GtkImage *image = GTK_IMAGE(gtk_image_new());
  gtk_image_set_pixel_size(image, 24);
  gtk_box_append(row, GTK_WIDGET(label));
  gtk_box_append(row, GTK_WIDGET(image));
  return (weather_row_t){.wrapper = row, .image = image, .label = label};
}

void init_weather(void) {
  weather_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(weather_window), "WeatherWindow");
  gtk_widget_add_css_class(GTK_WIDGET(weather_window), "widget-weather");

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  gtk_window_set_child(weather_window, GTK_WIDGET(layout));

  GtkBox *left_side = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(left_side), "weather-left-side");
  gtk_box_append(layout, GTK_WIDGET(left_side));

  gtk_box_append(left_side, gtk_label_new("Hourly"));
  for (size_t i = 0; i < WEATHER_HOURLY_ROWS_COUNT; i++) {
    weather_row_t row = weather_row_new();
    gtk_box_append(left_side, GTK_WIDGET(row.wrapper));
    weather_hourly_rows[i] = row;
  }

  GtkBox *right_side = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(right_side), "weather-right-side");
  gtk_box_append(layout, GTK_WIDGET(right_side));

  gtk_box_append(right_side, gtk_label_new("Daily"));
  for (size_t i = 0; i < WEATHER_DAILY_ROWS_COUNT; i++) {
    weather_row_t row = weather_row_new();
    gtk_box_append(right_side, GTK_WIDGET(row.wrapper));
    weather_daily_rows[i] = row;
  }
}

void toggle_window(GtkWindow *w) {
  gtk_widget_set_visible(GTK_WIDGET(w), !gtk_widget_get_visible(GTK_WIDGET(w)));
}

void toggle_launcher_window(void) {
  if (gtk_widget_get_visible(GTK_WIDGET(launcher_window)) == false) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListReset});
    gtk_editable_set_text(GTK_EDITABLE(launcher_input), "");
  }
  toggle_window(launcher_window);
}

void on_io_event(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case Workspaces: {
    for (size_t idx = 1; idx <= 10; idx++) {
      GtkButton *button = workspace_buttons[idx - 1];
      bool visible = false;
      for (size_t i = 0; i < event->workspaces.ids.len; i++) {
        if (event->workspaces.ids.ptr[i] == idx) {
          visible = true;
        }
      }
      gtk_widget_set_visible(GTK_WIDGET(button), visible || idx <= 5);
      gtk_widget_remove_css_class(GTK_WIDGET(button), "active");
      gtk_widget_remove_css_class(GTK_WIDGET(button), "inactive");
      if (idx == event->workspaces.active_id) {
        gtk_widget_add_css_class(GTK_WIDGET(button), "active");
      } else {
        gtk_widget_add_css_class(GTK_WIDGET(button), "inactive");
      }
    }
    break;
  }
  case Language: {
    if (strcmp(event->language.lang.ptr, "English (US)") == 0) {
      gtk_label_set_label(language_label, "EN");
    } else if (strcmp(event->language.lang.ptr, "Polish") == 0) {
      gtk_label_set_label(language_label, "PL");
    } else {
      gtk_label_set_label(language_label, "??");
    }
    break;
  }
  case Volume: {
    float volume = event->volume.volume;
    gtk_range_set_value(GTK_RANGE(sound_scale), volume);
    char *icon = NULL;
    if (volume == 0.0) {
      icon = "audio-volume-muted-symbolic";
    } else if (volume > 0.01 && volume < 0.34) {
      icon = "audio-volume-low-symbolic";
    } else if (volume > 0.34 && volume < 0.67) {
      icon = "audio-volume-medium-symbolic";
    } else if (volume > 0.67 && volume < 1.0) {
      icon = "audio-volume-high-symbolic";
    } else {
      icon = "audio-volume-overamplified-symbolic";
    }
    gtk_image_set_from_icon_name(sound_image, icon);
    break;
  }
  case CpuUsage: {
    for (size_t idx = 0; idx < 12; idx++) {
      GtkLabel *label = cpu_labels[idx];
      size_t load = event->cpu_usage.usage_per_core.ptr[idx];
      size_t indicator_idx =
          (size_t)((double)load / 100.0 * (double)CPU_INDICATORS_COUNT);

      if (indicator_idx == CPU_INDICATORS_COUNT) {
        indicator_idx -= 1;
      }

      const char *markup = CPU_INDICATORS[indicator_idx];
      gtk_label_set_label(label, markup);
    }
    break;
  }
  case Memory: {
    char buffer[100];
    sprintf(buffer, "RAM %.1fG/%.1fG", event->memory.used, event->memory.total);
    gtk_label_set_label(ram_label, buffer);
    break;
  }
  case WiFiStatus: {
    if (event->wi_fi_status.ssid.ptr == NULL) {
      gtk_label_set_label(network_label, "Not connected");
      gtk_widget_set_visible(GTK_WIDGET(network_image), false);
    } else {
      gtk_widget_set_visible(GTK_WIDGET(network_image), true);
      char buffer[100];
      sprintf(buffer, "%s (%d)%% ", event->wi_fi_status.ssid.ptr,
              event->wi_fi_status.strength);
      gtk_label_set_label(network_label, buffer);
    }
    break;
  }
  case Time: {
    gtk_label_set_label(time_label, event->time.time.ptr);
    gtk_widget_set_tooltip_text(GTK_WIDGET(time_label), event->time.date.ptr);
    break;
  }
  case CurrentWeather: {
    char buffer[100];
    sprintf(buffer, "%.1f℃ %s", event->current_weather.temperature,
            weather_code_to_description(event->current_weather.code));
    gtk_label_set_label(weather_label, buffer);
    break;
  }
  case ToggleSessionScreen: {
    toggle_window(session_window);
    break;
  }
  case ToggleLauncher: {
    toggle_launcher_window();
    break;
  }
  case AppList: {
    LAYER_SHELL_IO_CArray_App apps = event->app_list.apps;
    for (size_t i = 0; i < 5; i++) {
      launcher_row_t row = launcher_rows[i];
      if (i < apps.len) {
        LAYER_SHELL_IO_App app = apps.ptr[i];
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), true);
        if (app.selected) {
          gtk_widget_add_css_class(GTK_WIDGET(row.wrapper), "active");
        } else {
          gtk_widget_remove_css_class(GTK_WIDGET(row.wrapper), "active");
        }

        if (app.icon.tag == IconName) {
          gtk_image_set_from_icon_name(row.icon, app.icon.icon_name.ptr);
        } else {
          gtk_image_set_from_file(row.icon, app.icon.icon_path.ptr);
        }
        gtk_label_set_label(row.label, app.name.ptr);
      } else {
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), false);
      }
    }
    break;
  }
  case NetworkList: {
    LAYER_SHELL_IO_CArray_Network networks = event->network_list.list;
    for (size_t i = 0; i < 5; i++) {
      network_row_t row = networks_rows[i];
      if (i < networks.len) {
        LAYER_SHELL_IO_Network network = networks.ptr[i];
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), true);
        char buffer[100];
        sprintf(buffer, "%s: %s", network.iface.ptr, network.address.ptr);
        gtk_label_set_label(row.label, buffer);
        gtk_widget_set_tooltip_text(GTK_WIDGET(row.label), network.address.ptr);
      } else {
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), false);
      }
    }
    break;
  }
  case ForecastWeather: {
    LAYER_SHELL_IO_CArray_WeatherOnDay daily = event->forecast_weather.daily;
    LAYER_SHELL_IO_CArray_WeatherOnHour hourly = event->forecast_weather.hourly;

    for (size_t i = 0; i < WEATHER_HOURLY_ROWS_COUNT; i++) {
      LAYER_SHELL_IO_WeatherOnHour weather = hourly.ptr[i];
      weather_row_t row = weather_hourly_rows[i];

      char buffer[100];
      sprintf(buffer, "%s' %5.1f℃", weather.hour.ptr, weather.temperature);
      gtk_label_set_label(row.label, buffer);
      gtk_widget_set_tooltip_text(GTK_WIDGET(row.label),
                                  weather_code_to_description(weather.code));

      gtk_image_set_from_gicon(row.image, weather_code_to_icon(weather.code));
    }

    for (size_t i = 0; i < WEATHER_DAILY_ROWS_COUNT; i++) {
      LAYER_SHELL_IO_WeatherOnDay weather = daily.ptr[i];
      weather_row_t row = weather_daily_rows[i];

      char buffer[100];
      sprintf(buffer, "%s: %5.1f℃ - %5.1f℃", weather.day.ptr,
              weather.temperature_min, weather.temperature_max);
      gtk_label_set_label(row.label, buffer);
      gtk_widget_set_tooltip_text(GTK_WIDGET(row.label),
                                  weather_code_to_description(weather.code));

      gtk_image_set_from_gicon(row.image, weather_code_to_icon(weather.code));
    }
    break;
  }
  }
}

int poll_events(void) {
  layer_shell_io_poll_events();
  return 1;
}

void init_widgets(void) {
  init_top_bar();
  init_session_screen();
  init_launcher();
  init_networks();
  init_htop();
  init_weather();

  layer_shell_io_subscribe(on_io_event);

  g_timeout_add(50, G_SOURCE_FUNC(poll_events), NULL);

  printf("Finished building widgets...\n");
}

void on_workspace_button_click(GtkButton *self, gpointer data) {
  size_t idx = (size_t)data;
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = HyprlandGoToWorkspace, .hyprland_go_to_workspace = {idx}});
}

void open_htop_window() { toggle_window(htop_window); }

void on_sound_scale_changed() {
  GtkAdjustment *adj = gtk_range_get_adjustment(GTK_RANGE(sound_scale));
  double value = CLAMP(gtk_adjustment_get_value(adj), 0.0, 1.0);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = SetVolume, .set_volume = {.volume = value}});
}

void spawn_system_monitor() {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnSystemMonitor});
}

void open_networks_window() { toggle_window(networks_window); }

void open_sessions_window() { toggle_window(session_window); }

void open_weather_window() { toggle_window(weather_window); }

void activate_top_bar(GApplication *app) {
  gtk_window_set_application(top_bar_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(top_bar_window);
  gtk_layer_set_layer(top_bar_window, GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(top_bar_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(top_bar_window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(top_bar_window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(top_bar_window, GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(top_bar_window, "LayerShell/TopBar");

  for (size_t idx = 0; idx < 10; idx++) {
    GtkButton *button = workspace_buttons[idx];
    g_signal_connect(button, "clicked", G_CALLBACK(on_workspace_button_click),
                     (void *)idx);
  }

  g_signal_connect(htop_widget, "clicked", open_htop_window, NULL);

  GtkGestureClick *sound_ctrl = GTK_GESTURE_CLICK(gtk_gesture_click_new());
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(sound_ctrl),
                                             GTK_PHASE_CAPTURE);
  g_signal_connect(sound_ctrl, "released", on_sound_scale_changed, NULL);
  gtk_widget_add_controller(GTK_WIDGET(sound_widget),
                            GTK_EVENT_CONTROLLER(sound_ctrl));

  g_signal_connect(ram_widget, "clicked", spawn_system_monitor, NULL);

  g_signal_connect(network_widget, "clicked", open_networks_window, NULL);

  g_signal_connect(session_widget, "clicked", open_sessions_window, NULL);

  g_signal_connect(weather_widget, "clicked", open_weather_window, NULL);

  gtk_window_present(top_bar_window);
}

void session_lock(void) {
  toggle_window(session_window);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Lock});
}
void session_reboot(void) {
  toggle_window(session_window);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Reboot});
}
void session_shutdown(void) {
  toggle_window(session_window);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Shutdown});
}
void session_logout(void) {
  toggle_window(session_window);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Logout});
}

void on_session_window_key_press(GtkEventControllerKey *self, guint keyval,
                                 guint keycode, GdkModifierType state,
                                 gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_window(session_window);
  }
}

void activate_session_screen(GApplication *app) {
  gtk_window_set_application(session_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(session_window);
  gtk_layer_set_layer(session_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(session_window, "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(session_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(lock_button, "clicked", session_lock, NULL);
  g_signal_connect(reboot_button, "clicked", session_reboot, NULL);
  g_signal_connect(shutdown_button, "clicked", session_shutdown, NULL);
  g_signal_connect(logout_button, "clicked", session_logout, NULL);

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(on_session_window_key_press),
                   NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(session_window),
                            GTK_EVENT_CONTROLLER(ctrl));

  gtk_window_present(session_window);
  gtk_widget_set_visible(GTK_WIDGET(session_window), false);
}

void launcher_exec_selected(void) {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListExecSelected});
  toggle_launcher_window();
}

void launcher_input_changed(GtkEditable *editable) {
  const unsigned char *search =
      (const unsigned char *)gtk_editable_get_text(editable);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = AppListSetSearch, .app_list_set_search = {.search = search}});
}

gboolean on_launcher_window_key_press(GtkEventControllerKey *self, guint keyval,
                                      guint keycode, GdkModifierType state,
                                      gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_launcher_window();
  } else if (strcmp(gdk_keyval_name(keyval), "Up") == 0) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListGoUp});
  } else if (strcmp(gdk_keyval_name(keyval), "Down") == 0) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListGoDown});
  }

  return false;
}

void activate_launcher(GApplication *app) {
  gtk_window_set_application(launcher_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(launcher_window);
  gtk_layer_set_layer(launcher_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(launcher_window, "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(launcher_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(launcher_input, "activate", launcher_exec_selected, NULL);
  g_signal_connect(launcher_input, "changed",
                   G_CALLBACK(launcher_input_changed), NULL);

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed",
                   G_CALLBACK(on_launcher_window_key_press), NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(launcher_window),
                            GTK_EVENT_CONTROLLER(ctrl));

  gtk_window_present(launcher_window);
  gtk_widget_set_visible(GTK_WIDGET(launcher_window), false);
}

void on_networks_window_key_press(GtkEventControllerKey *self, guint keyval,
                                  guint keycode, GdkModifierType state,
                                  gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_window(networks_window);
  }
}

void set_on_network_row_click(network_row_t row, GCallback callback,
                              void *data) {
  GtkGestureClick *ctrl = GTK_GESTURE_CLICK(gtk_gesture_click_new());
  g_signal_connect(ctrl, "pressed", callback, data);
  gtk_widget_add_controller(GTK_WIDGET(row.wrapper),
                            GTK_EVENT_CONTROLLER(ctrl));
}

void on_network_settings_row_click(void) {
  toggle_window(networks_window);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnNetworkEditor});
}
void on_network_exit_row_click(void) { toggle_window(networks_window); }

typedef struct {
  size_t row_idx;
  char *text;
} network_row_safe_point_t;

network_row_safe_point_t *network_row_safe_point_new(size_t row_idx,
                                                     const char *text) {
  size_t len = strlen(text);
  char *copy = malloc(len + 1);
  memcpy(copy, text, len);
  copy[len] = 0;

  network_row_safe_point_t *safepoint =
      malloc(sizeof(network_row_safe_point_t));
  safepoint->row_idx = row_idx;
  safepoint->text = copy;
  return safepoint;
}

void network_row_safe_point_free(network_row_safe_point_t *safepoint) {
  free(safepoint->text);
  free(safepoint);
}

void network_row_restore_label(gpointer user_data) {
  network_row_safe_point_t *safepoint = (network_row_safe_point_t *)user_data;
  GtkLabel *label = networks_rows[safepoint->row_idx].label;
  gtk_label_set_label(label, safepoint->text);
  network_row_safe_point_free(safepoint);
}

void on_network_row_click(GtkGestureClick *self, gint n_press, gdouble x,
                          gdouble y, gpointer user_data) {
  size_t row_idx = (size_t)(user_data);
  network_row_t row = networks_rows[row_idx];
  const char *ip = gtk_widget_get_tooltip_text(GTK_WIDGET(row.label));
  const char *label = gtk_label_get_label(row.label);
  network_row_safe_point_t *safepoint =
      network_row_safe_point_new(row_idx, label);

  GdkDisplay *display = gdk_display_get_default();
  GdkClipboard *clipboard = gdk_display_get_clipboard(display);
  gdk_clipboard_set_text(clipboard, ip);

  gtk_label_set_label(row.label, "Copied!");
  g_timeout_add_seconds_once(1, network_row_restore_label, safepoint);
}

void activate_networks(GApplication *app) {
  gtk_window_set_application(networks_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(networks_window);
  gtk_layer_set_layer(networks_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(networks_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(networks_window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(networks_window, GTK_LAYER_SHELL_EDGE_TOP, 50);
  gtk_layer_set_namespace(networks_window, "LayerShell/Networks");
  gtk_layer_set_keyboard_mode(networks_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  set_on_network_row_click(network_settings_row,
                           G_CALLBACK(on_network_settings_row_click), NULL);
  set_on_network_row_click(network_exit_row,
                           G_CALLBACK(on_network_exit_row_click), NULL);

  for (size_t i = 0; i < 5; i++) {
    network_row_t row = networks_rows[i];
    set_on_network_row_click(row, G_CALLBACK(on_network_row_click), (void *)i);
  }

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed",
                   G_CALLBACK(on_networks_window_key_press), NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(networks_window),
                            GTK_EVENT_CONTROLLER(ctrl));
}

void on_htop_window_key_press(GtkEventControllerKey *self, guint keyval,
                              guint keycode, GdkModifierType state,
                              gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_window(htop_window);
  }
}

void activate_htop(GApplication *app) {
  gtk_window_set_application(htop_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(htop_window);
  gtk_layer_set_layer(htop_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(htop_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(htop_window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(htop_window, GTK_LAYER_SHELL_EDGE_TOP, 50);
  gtk_layer_set_margin(htop_window, GTK_LAYER_SHELL_EDGE_RIGHT, 600);
  gtk_layer_set_namespace(htop_window, "LayerShell/Htop");
  gtk_layer_set_keyboard_mode(htop_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  VteTerminal *terminal = VTE_TERMINAL(vte_terminal_new());
  const char *home = getenv("HOME");
  char *argv[] = {"htop", NULL};
  vte_terminal_spawn_async(terminal, VTE_PTY_DEFAULT, home, argv, NULL,
                           G_SPAWN_DEFAULT, NULL, NULL, NULL, -1, NULL, NULL,
                           NULL);
  gtk_window_set_child(htop_window, GTK_WIDGET(terminal));

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(on_htop_window_key_press),
                   NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(htop_window),
                            GTK_EVENT_CONTROLLER(ctrl));
}

void on_weather_window_key_press(GtkEventControllerKey *self, guint keyval,
                                 guint keycode, GdkModifierType state,
                                 gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_window(weather_window);
  }
}

void activate_weather(GApplication *app) {
  gtk_window_set_application(weather_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(weather_window);
  gtk_layer_set_layer(weather_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(weather_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(weather_window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(weather_window, GTK_LAYER_SHELL_EDGE_TOP, 50);
  gtk_layer_set_margin(weather_window, GTK_LAYER_SHELL_EDGE_RIGHT, 750);
  gtk_layer_set_namespace(weather_window, "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(weather_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(on_weather_window_key_press),
                   NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(weather_window),
                            GTK_EVENT_CONTROLLER(ctrl));
}

#endif // WIDGETS_H
