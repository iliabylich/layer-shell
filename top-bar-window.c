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

#define ns(name) top_bar_ns_##name

static GtkWindow *ns(window);

static GtkWidget *ns(worspaces);
static GtkWidget *ns(workspace_buttons)[10];

static GtkWidget *ns(htop);

static GtkWidget *ns(weather);
static GtkWidget *ns(weather_label);

static GtkWidget *ns(language);
static GtkWidget *ns(language_label);

static GtkWidget *ns(sound);
static GtkWidget *ns(sound_image);
static GtkWidget *ns(sound_scale);

static GtkWidget *ns(cpu);
static GtkWidget *ns(cpu_labels)[12];
#define CPU_INDICATORS_COUNT 8
static const char *CPU_INDICATORS[CPU_INDICATORS_COUNT] = {
    "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
};

static GtkWidget *ns(ram);
static GtkWidget *ns(ram_label);

static GtkWidget *ns(network);
static GtkWidget *ns(network_label);
static GtkWidget *ns(network_image);

static GtkWidget *ns(time);
static GtkWidget *ns(time_label);

static GtkWidget *ns(session);

static void ns(init)(void) {
  ns(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(ns(window)), "TopBarWindow");

  GtkWidget *layout = gtk_center_box_new();
  gtk_widget_add_css_class(layout, "main-wrapper");
  gtk_window_set_child(ns(window), layout);

  GtkWidget *left = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
  gtk_center_box_set_start_widget(GTK_CENTER_BOX(layout), left);

  GtkWidget *right = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(layout), right);

  // workspaces
  ns(worspaces) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_widget_add_css_class(ns(worspaces), "widget");
  gtk_widget_add_css_class(ns(worspaces), "workspaces");
  for (size_t i = 0; i < 10; i++) {
    GtkWidget *button = gtk_button_new();
    char buffer[3];
    sprintf(buffer, "%lu", i + 1);
    GtkWidget *label = gtk_label_new(buffer);
    gtk_button_set_child(GTK_BUTTON(button), label);
    gtk_box_append(GTK_BOX(ns(worspaces)), button);
    ns(workspace_buttons)[i] = button;
  }
  gtk_box_append(GTK_BOX(left), ns(worspaces));

  // htop
  ns(htop) = gtk_button_new();
  gtk_widget_add_css_class(ns(htop), "widget");
  gtk_widget_add_css_class(ns(htop), "terminal");
  gtk_widget_add_css_class(ns(htop), "padded");
  gtk_widget_add_css_class(ns(htop), "clickable");
  GtkWidget *htop_label = gtk_label_new("Htop");
  gtk_button_set_child(GTK_BUTTON(ns(htop)), htop_label);
  gtk_box_append(GTK_BOX(right), ns(htop));

  // weather
  ns(weather_label) = gtk_label_new("--");
  ns(weather) = gtk_button_new();
  gtk_widget_add_css_class(ns(weather), "widget");
  gtk_widget_add_css_class(ns(weather), "weather");
  gtk_widget_add_css_class(ns(weather), "padded");
  gtk_widget_add_css_class(ns(weather), "clickable");
  gtk_button_set_child(GTK_BUTTON(ns(weather)), ns(weather_label));
  gtk_box_append(GTK_BOX(right), ns(weather));

  // language
  ns(language_label) = gtk_label_new("--");
  ns(language) = gtk_center_box_new();
  gtk_widget_add_css_class(ns(language), "widget");
  gtk_widget_add_css_class(ns(language), "language");
  gtk_widget_add_css_class(ns(language), "padded");
  gtk_center_box_set_center_widget(GTK_CENTER_BOX(ns(language)),
                                   ns(language_label));
  gtk_box_append(GTK_BOX(right), ns(language));

  // sound
  ns(sound) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
  gtk_widget_add_css_class(ns(sound), "widget");
  gtk_widget_add_css_class(ns(sound), "sound");
  gtk_widget_add_css_class(ns(sound), "padded");
  ns(sound_image) = gtk_image_new();
  gtk_image_set_from_icon_name(GTK_IMAGE(ns(sound_image)), "dialog-question");
  gtk_box_append(GTK_BOX(ns(sound)), ns(sound_image));
  ns(sound_scale) =
      gtk_scale_new(GTK_ORIENTATION_HORIZONTAL,
                    gtk_adjustment_new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0));
  gtk_widget_add_css_class(ns(sound_scale), "sound-slider");
  gtk_box_append(GTK_BOX(ns(sound)), ns(sound_scale));
  gtk_box_append(GTK_BOX(right), ns(sound));

  // cpu
  ns(cpu) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 3);
  gtk_widget_add_css_class(ns(cpu), "widget");
  gtk_widget_add_css_class(ns(cpu), "cpu");
  gtk_widget_add_css_class(ns(cpu), "padded");
  for (size_t i = 0; i < 12; i++) {
    GtkWidget *label = gtk_label_new(NULL);
    gtk_label_set_use_markup(GTK_LABEL(label), true);
    gtk_box_append(GTK_BOX(ns(cpu)), label);
    ns(cpu_labels)[i] = label;
  }
  gtk_box_append(GTK_BOX(right), ns(cpu));

  // ram
  ns(ram_label) = gtk_label_new(NULL);
  ns(ram) = gtk_button_new();
  gtk_widget_add_css_class(ns(ram), "widget");
  gtk_widget_add_css_class(ns(ram), "memory");
  gtk_widget_add_css_class(ns(ram), "padded");
  gtk_widget_add_css_class(ns(ram), "clickable");
  gtk_button_set_child(GTK_BUTTON(ns(ram)), ns(ram_label));
  gtk_box_append(GTK_BOX(right), ns(ram));

  // network
  ns(network_label) = gtk_label_new("--");
  ns(network_image) = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(ns(network_image)), get_icon(WIFI_ICON));
  GtkWidget *network_wrapper = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_box_append(GTK_BOX(network_wrapper), ns(network_label));
  gtk_box_append(GTK_BOX(network_wrapper), ns(network_image));
  ns(network) = gtk_button_new();
  gtk_widget_add_css_class(ns(network), "widget");
  gtk_widget_add_css_class(ns(network), "network");
  gtk_widget_add_css_class(ns(network), "padded");
  gtk_widget_add_css_class(ns(network), "clickable");
  gtk_widget_set_cursor(ns(network), gdk_cursor_new_from_name("pointer", NULL));
  gtk_button_set_child(GTK_BUTTON(ns(network)), network_wrapper);
  gtk_box_append(GTK_BOX(right), ns(network));

  // clock
  ns(time_label) = gtk_label_new("--");
  ns(time) = gtk_center_box_new();
  gtk_widget_add_css_class(ns(time), "widget");
  gtk_widget_add_css_class(ns(time), "clock");
  gtk_widget_add_css_class(ns(time), "padded");
  gtk_center_box_set_center_widget(GTK_CENTER_BOX(ns(time)), ns(time_label));
  gtk_box_append(GTK_BOX(right), ns(time));

  // session
  ns(session) = gtk_button_new();
  gtk_widget_add_css_class(ns(session), "widget");
  gtk_widget_add_css_class(ns(session), "power");
  gtk_widget_add_css_class(ns(session), "padded");
  gtk_widget_add_css_class(ns(session), "clickable");
  gtk_widget_set_cursor(ns(network), gdk_cursor_new_from_name("pointer", NULL));
  GtkWidget *session_image = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(session_image), get_icon(POWER_ICON));
  gtk_button_set_child(GTK_BUTTON(ns(session)), session_image);
  gtk_box_append(GTK_BOX(right), ns(session));
}

static void ns(workspace_button_on_click)(GtkButton *, gpointer data) {
  size_t idx = (size_t)data;
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = HyprlandGoToWorkspace, .hyprland_go_to_workspace = {idx}});
}

static void ns(sound_scale_on_change)(void) {
  GtkAdjustment *adj = gtk_range_get_adjustment(GTK_RANGE(ns(sound_scale)));
  double value = CLAMP(gtk_adjustment_get_value(adj), 0.0, 1.0);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = SetVolume, .set_volume = {.volume = value}});
}

static void ns(spawn_system_monitor)(void) {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnSystemMonitor});
}

static void ns(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case Workspaces: {
    for (size_t idx = 1; idx <= 10; idx++) {
      GtkWidget *button = ns(workspace_buttons)[idx - 1];
      bool visible = false;
      for (size_t i = 0; i < event->workspaces.ids.len; i++) {
        if (event->workspaces.ids.ptr[i] == idx) {
          visible = true;
        }
      }
      gtk_widget_set_visible(button, visible || idx <= 5);
      gtk_widget_remove_css_class(button, "active");
      gtk_widget_remove_css_class(button, "inactive");
      if (idx == event->workspaces.active_id) {
        gtk_widget_add_css_class(button, "active");
      } else {
        gtk_widget_add_css_class(button, "inactive");
      }
    }
    break;
  }
  case Language: {
    if (strcmp(event->language.lang.ptr, "English (US)") == 0) {
      gtk_label_set_label(GTK_LABEL(ns(language_label)), "EN");
    } else if (strcmp(event->language.lang.ptr, "Polish") == 0) {
      gtk_label_set_label(GTK_LABEL(ns(language_label)), "PL");
    } else {
      gtk_label_set_label(GTK_LABEL(ns(language_label)), "??");
    }
    break;
  }
  case Volume: {
    float volume = event->volume.volume;
    gtk_range_set_value(GTK_RANGE(ns(sound_scale)), volume);
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
    gtk_image_set_from_icon_name(GTK_IMAGE(ns(sound_image)), icon);
    break;
  }
  case CpuUsage: {
    for (size_t idx = 0; idx < 12; idx++) {
      GtkWidget *label = ns(cpu_labels)[idx];
      size_t load = event->cpu_usage.usage_per_core.ptr[idx];
      size_t indicator_idx =
          (size_t)((double)load / 100.0 * (double)CPU_INDICATORS_COUNT);

      if (indicator_idx == CPU_INDICATORS_COUNT) {
        indicator_idx -= 1;
      }

      const char *markup = CPU_INDICATORS[indicator_idx];
      gtk_label_set_label(GTK_LABEL(label), markup);
    }
    break;
  }
  case Memory: {
    char buffer[100];
    sprintf(buffer, "RAM %.1fG/%.1fG", event->memory.used, event->memory.total);
    gtk_label_set_label(GTK_LABEL(ns(ram_label)), buffer);
    break;
  }
  case WiFiStatus: {
    if (event->wi_fi_status.ssid.ptr == NULL) {
      gtk_widget_set_visible(ns(network_image), false);
      gtk_label_set_label(GTK_LABEL(ns(network_label)), "Not connected");
    } else {
      gtk_widget_set_visible(ns(network_image), true);
      char buffer[100];
      sprintf(buffer, "%s (%d)%% ", event->wi_fi_status.ssid.ptr,
              event->wi_fi_status.strength);
      gtk_label_set_label(GTK_LABEL(ns(network_label)), buffer);
    }
    break;
  }
  case Time: {
    gtk_label_set_label(GTK_LABEL(ns(time_label)), event->time.time.ptr);
    gtk_widget_set_tooltip_text(ns(time_label), event->time.date.ptr);
    break;
  }
  case CurrentWeather: {
    char buffer[100];
    sprintf(buffer, "%.1f℃ %s", event->current_weather.temperature,
            weather_code_to_description(event->current_weather.code));
    gtk_label_set_label(GTK_LABEL(ns(weather_label)), buffer);
    break;
  }

  default:
    break;
  }
}

static bool ns(bottom_right_point_of)(GtkWidget *widget,
                                      graphene_point_t *out) {
  graphene_rect_t bounds;
  if (!gtk_widget_compute_bounds(widget, GTK_WIDGET(ns(window)), &bounds)) {
    return false;
  }

  out->x = bounds.origin.x + bounds.size.width;
  out->y = bounds.origin.y + bounds.size.height;

  return true;
}

static void ns(htop_btn_on_click)() {
  graphene_point_t bottom_right;
  if (!ns(bottom_right_point_of)(ns(htop), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the htop widget");
    return;
  }
  uint32_t margin_left = bottom_right.x - HTOP.width() / 2.0;
  uint32_t margin_top = bottom_right.y;
  HTOP.move(margin_left, margin_top);

  HTOP.toggle();
}

static void ns(weather_btn_on_click)() {
  graphene_point_t bottom_right;
  if (!ns(bottom_right_point_of)(ns(weather), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the weather widget");
    return;
  }
  uint32_t margin_left = bottom_right.x - WEATHER.width();
  uint32_t margin_top = bottom_right.y;
  WEATHER.move(margin_left, margin_top);

  WEATHER.toggle();
}

static void ns(network_btn_on_click)() {
  graphene_point_t bottom_right;
  if (!ns(bottom_right_point_of)(ns(network), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the network widget");
    return;
  }
  uint32_t margin_left = bottom_right.x - NETWORK.width();
  uint32_t margin_top = bottom_right.y;
  NETWORK.move(margin_left, margin_top);

  NETWORK.toggle();
}

static void ns(activate)(GApplication *app) {
  gtk_window_set_application(ns(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(ns(window));
  gtk_layer_set_layer(ns(window), GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(ns(window), GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(ns(window), "LayerShell/TopBar");

  for (size_t idx = 0; idx < 10; idx++) {
    GtkWidget *button = ns(workspace_buttons)[idx];
    g_signal_connect(button, "clicked",
                     G_CALLBACK(ns(workspace_button_on_click)), (void *)idx);
  }

  g_signal_connect(ns(htop), "clicked", ns(htop_btn_on_click), NULL);

  g_signal_connect(ns(weather), "clicked", ns(weather_btn_on_click), NULL);

  GtkGestureClick *sound_ctrl = GTK_GESTURE_CLICK(gtk_gesture_click_new());
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(sound_ctrl),
                                             GTK_PHASE_CAPTURE);
  g_signal_connect(sound_ctrl, "released", ns(sound_scale_on_change), NULL);
  gtk_widget_add_controller(ns(sound), GTK_EVENT_CONTROLLER(sound_ctrl));

  g_signal_connect(ns(ram), "clicked", ns(spawn_system_monitor), NULL);

  g_signal_connect(ns(network), "clicked", ns(network_btn_on_click), NULL);

  g_signal_connect(ns(session), "clicked", SESSION.toggle, NULL);

  layer_shell_io_subscribe(ns(on_io_event));

  gtk_window_present(ns(window));
}

window_t TOP_BAR = {.init = ns(init), .activate = ns(activate)};
