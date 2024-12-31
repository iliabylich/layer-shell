#include "top-bar-window.h"
#include "bindings.h"
#include "htop-window.h"
#include "icons.h"
#include "network-window.h"
#include "session-window.h"
#include "weather-helper.h"
#include "weather-window.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

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

void init_top_bar_window(void) {
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
  gtk_image_set_from_gicon(network_image, get_icon(WIFI_ICON));
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
  gtk_image_set_from_gicon(session_image, get_icon(POWER_ICON));
  gtk_button_set_child(session_widget, GTK_WIDGET(session_image));
  gtk_box_append(right, GTK_WIDGET(session_widget));
}

static void on_workspace_button_click(__attribute__((unused)) GtkButton *self,
                                      gpointer data) {
  size_t idx = (size_t)data;
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = HyprlandGoToWorkspace, .hyprland_go_to_workspace = {idx}});
}

static void on_sound_scale_changed(void) {
  GtkAdjustment *adj = gtk_range_get_adjustment(GTK_RANGE(sound_scale));
  double value = CLAMP(gtk_adjustment_get_value(adj), 0.0, 1.0);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = SetVolume, .set_volume = {.volume = value}});
}

static void spawn_system_monitor(void) {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnSystemMonitor});
}

static void top_bar_window_on_event(const LAYER_SHELL_IO_Event *event) {
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

  default:
    break;
  }
}

void activate_top_bar_window(GApplication *app) {
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

  g_signal_connect(htop_widget, "clicked", toggle_htop_window, NULL);

  GtkGestureClick *sound_ctrl = GTK_GESTURE_CLICK(gtk_gesture_click_new());
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(sound_ctrl),
                                             GTK_PHASE_CAPTURE);
  g_signal_connect(sound_ctrl, "released", on_sound_scale_changed, NULL);
  gtk_widget_add_controller(GTK_WIDGET(sound_widget),
                            GTK_EVENT_CONTROLLER(sound_ctrl));

  g_signal_connect(ram_widget, "clicked", spawn_system_monitor, NULL);

  g_signal_connect(network_widget, "clicked", toggle_network_window, NULL);

  g_signal_connect(session_widget, "clicked", toggle_session_window, NULL);

  g_signal_connect(weather_widget, "clicked", toggle_weather_window, NULL);

  layer_shell_io_subscribe(top_bar_window_on_event);

  gtk_window_present(top_bar_window);
}
