#include "top-bar-window.h"
#include "bindings.h"
#include "gtk/gtkshortcut.h"
#include "htop-widget.h"
#include "icons.h"
#include "language-widget.h"
#include "network-window.h"
#include "session-window.h"
#include "weather-widget.h"
#include "workspaces-widget.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define _(name) top_bar_ns_##name

static GtkWindow *_(window);

static GtkWidget *_(sound);
static GtkWidget *_(sound_image);
static GtkWidget *_(sound_scale);

static GtkWidget *_(cpu);
static GtkWidget *_(cpu_labels)[12];
#define CPU_INDICATORS_COUNT 8
static const char *CPU_INDICATORS[CPU_INDICATORS_COUNT] = {
    "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
};

static GtkWidget *_(ram);
static GtkWidget *_(ram_label);

static GtkWidget *_(network);
static GtkWidget *_(network_label);
static GtkWidget *_(network_image);
static GtkWidget *_(download_speed_label);
static GtkWidget *_(download_speed_icon);
static GtkWidget *_(upload_speed_label);
static GtkWidget *_(upload_speed_icon);

static GtkWidget *_(time);
static GtkWidget *_(time_label);

static GtkWidget *_(session);

static void _(init)(void) {
  _(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(_(window)), "TopBarWindow");

  GtkWidget *layout = gtk_center_box_new();
  gtk_widget_add_css_class(layout, "main-wrapper");
  gtk_window_set_child(_(window), layout);

  GtkWidget *left = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
  gtk_center_box_set_start_widget(GTK_CENTER_BOX(layout), left);

  GtkWidget *right = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(layout), right);

  // workspaces
  WORKSPACES_WIDGET.init();
  gtk_box_append(GTK_BOX(left), WORKSPACES_WIDGET.main_widget());

  // htop
  HTOP_WIDGET.init();
  gtk_box_append(GTK_BOX(right), HTOP_WIDGET.main_widget());

  // weather
  WEATHER_WIDGET.init();
  gtk_box_append(GTK_BOX(right), WEATHER_WIDGET.main_widget());

  // language
  LANGUAGE_WIDGET.init();
  gtk_box_append(GTK_BOX(right), LANGUAGE_WIDGET.main_widget());

  // sound
  _(sound) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
  gtk_widget_add_css_class(_(sound), "widget");
  gtk_widget_add_css_class(_(sound), "sound");
  gtk_widget_add_css_class(_(sound), "padded");
  _(sound_image) = gtk_image_new();
  gtk_image_set_from_icon_name(GTK_IMAGE(_(sound_image)), "dialog-question");
  gtk_box_append(GTK_BOX(_(sound)), _(sound_image));
  _(sound_scale) =
      gtk_scale_new(GTK_ORIENTATION_HORIZONTAL,
                    gtk_adjustment_new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0));
  gtk_widget_add_css_class(_(sound_scale), "sound-slider");
  gtk_box_append(GTK_BOX(_(sound)), _(sound_scale));
  gtk_box_append(GTK_BOX(right), _(sound));

  // cpu
  _(cpu) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 3);
  gtk_widget_add_css_class(_(cpu), "widget");
  gtk_widget_add_css_class(_(cpu), "cpu");
  gtk_widget_add_css_class(_(cpu), "padded");
  for (size_t i = 0; i < 12; i++) {
    GtkWidget *label = gtk_label_new(NULL);
    gtk_label_set_use_markup(GTK_LABEL(label), true);
    gtk_box_append(GTK_BOX(_(cpu)), label);
    _(cpu_labels)[i] = label;
  }
  gtk_box_append(GTK_BOX(right), _(cpu));

  // ram
  _(ram_label) = gtk_label_new(NULL);
  _(ram) = gtk_button_new();
  gtk_widget_add_css_class(_(ram), "widget");
  gtk_widget_add_css_class(_(ram), "memory");
  gtk_widget_add_css_class(_(ram), "padded");
  gtk_widget_add_css_class(_(ram), "clickable");
  gtk_button_set_child(GTK_BUTTON(_(ram)), _(ram_label));
  gtk_box_append(GTK_BOX(right), _(ram));

  // network
  _(network_label) = gtk_label_new("--");
  _(network_image) = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(_(network_image)), get_wifi_icon());
  _(download_speed_label) = gtk_label_new("??");
  gtk_widget_add_css_class(_(download_speed_label), "network-speed-label");
  _(download_speed_icon) = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(_(download_speed_icon)),
                           get_download_speed_icon());
  _(upload_speed_label) = gtk_label_new("??");
  gtk_widget_add_css_class(_(upload_speed_label), "network-speed-label");
  _(upload_speed_icon) = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(_(upload_speed_icon)),
                           get_upload_speed_icon());

  GtkWidget *network_wrapper = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_box_append(GTK_BOX(network_wrapper), _(network_label));
  gtk_box_append(GTK_BOX(network_wrapper), _(network_image));

  GtkWidget *sep = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
  gtk_box_append(GTK_BOX(network_wrapper), sep);

  gtk_box_append(GTK_BOX(network_wrapper), _(download_speed_label));
  gtk_box_append(GTK_BOX(network_wrapper), _(download_speed_icon));
  gtk_box_append(GTK_BOX(network_wrapper), _(upload_speed_label));
  gtk_box_append(GTK_BOX(network_wrapper), _(upload_speed_icon));

  _(network) = gtk_button_new();
  gtk_widget_add_css_class(_(network), "widget");
  gtk_widget_add_css_class(_(network), "network");
  gtk_widget_add_css_class(_(network), "padded");
  gtk_widget_add_css_class(_(network), "clickable");
  gtk_widget_set_cursor(_(network), gdk_cursor_new_from_name("pointer", NULL));
  gtk_button_set_child(GTK_BUTTON(_(network)), network_wrapper);
  gtk_box_append(GTK_BOX(right), _(network));

  // clock
  _(time_label) = gtk_label_new("--");
  _(time) = gtk_center_box_new();
  gtk_widget_add_css_class(_(time), "widget");
  gtk_widget_add_css_class(_(time), "clock");
  gtk_widget_add_css_class(_(time), "padded");
  gtk_center_box_set_center_widget(GTK_CENTER_BOX(_(time)), _(time_label));
  gtk_box_append(GTK_BOX(right), _(time));

  // session
  _(session) = gtk_button_new();
  gtk_widget_add_css_class(_(session), "widget");
  gtk_widget_add_css_class(_(session), "power");
  gtk_widget_add_css_class(_(session), "padded");
  gtk_widget_add_css_class(_(session), "clickable");
  gtk_widget_set_cursor(_(network), gdk_cursor_new_from_name("pointer", NULL));
  GtkWidget *session_image = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(session_image), get_power_icon());
  gtk_button_set_child(GTK_BUTTON(_(session)), session_image);
  gtk_box_append(GTK_BOX(right), _(session));
}

static void _(sound_scale_on_change)(void) {
  GtkAdjustment *adj = gtk_range_get_adjustment(GTK_RANGE(_(sound_scale)));
  double value = CLAMP(gtk_adjustment_get_value(adj), 0.0, 1.0);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = SetVolume, .set_volume = {.volume = value}});
}

static void _(spawn_system_monitor)(void) {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnSystemMonitor});
}

static void _(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case Volume: {
    float volume = event->volume.volume;
    gtk_range_set_value(GTK_RANGE(_(sound_scale)), volume);
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
    gtk_image_set_from_icon_name(GTK_IMAGE(_(sound_image)), icon);
    break;
  }
  case CpuUsage: {
    for (size_t idx = 0; idx < 12; idx++) {
      GtkWidget *label = _(cpu_labels)[idx];
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
    gtk_label_set_label(GTK_LABEL(_(ram_label)), buffer);
    break;
  }
  case WifiStatus: {
    if (event->wifi_status.wifi_status.tag == None_WifiStatus) {
      gtk_widget_set_visible(_(network_image), false);
      gtk_label_set_label(GTK_LABEL(_(network_label)), "Not connected");
    } else {
      gtk_widget_set_visible(_(network_image), true);
      char buffer[100];
      sprintf(buffer, "%s (%d)%% ", event->wifi_status.wifi_status.some.ssid,
              event->wifi_status.wifi_status.some.strength);
      gtk_label_set_label(GTK_LABEL(_(network_label)), buffer);
    }
    break;
  }
  case NetworkSpeed: {
    gtk_label_set_label(GTK_LABEL(_(download_speed_label)),
                        event->network_speed.download_speed);
    gtk_label_set_label(GTK_LABEL(_(upload_speed_label)),
                        event->network_speed.upload_speed);
    break;
  }
  case Time: {
    gtk_label_set_label(GTK_LABEL(_(time_label)), event->time.time);
    gtk_widget_set_tooltip_text(_(time_label), event->time.date);
    break;
  }

  default:
    break;
  }
}

static void _(network_btn_on_click)(void) {
  graphene_point_t bottom_right;
  if (!bottom_right_point_of(_(network), TOP_BAR.window(), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the network widget");
    return;
  }
  int margin_left = bottom_right.x - NETWORK.width;
  int margin_top = bottom_right.y;
  NETWORK.move(margin_left, margin_top);

  NETWORK.toggle();
}

static GtkWindow *_(get_window)(void) { return _(window); }

static void _(activate)(GApplication *app) {
  gtk_window_set_application(_(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(_(window));
  gtk_layer_set_layer(_(window), GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(_(window), GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(_(window), "LayerShell/TopBar");

  WORKSPACES_WIDGET.activate();
  HTOP_WIDGET.activate();
  WEATHER_WIDGET.activate();
  LANGUAGE_WIDGET.activate();

  GtkEventController *sound_ctrl =
      GTK_EVENT_CONTROLLER(gtk_gesture_click_new());
  gtk_event_controller_set_propagation_phase(sound_ctrl, GTK_PHASE_CAPTURE);
  g_signal_connect(sound_ctrl, "released", _(sound_scale_on_change), NULL);
  gtk_widget_add_controller(_(sound), sound_ctrl);

  g_signal_connect(_(ram), "clicked", _(spawn_system_monitor), NULL);

  g_signal_connect(_(network), "clicked", _(network_btn_on_click), NULL);

  g_signal_connect(_(session), "clicked", SESSION.toggle, NULL);

  layer_shell_io_subscribe(_(on_io_event));

  gtk_window_present(_(window));
}

window_t TOP_BAR = {
    .init = _(init), .activate = _(activate), .window = _(get_window)};
