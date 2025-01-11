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

#define _(name) top_bar_ns_##name

static GtkWindow *_(window);

static GtkWidget *_(worspaces);
static GtkWidget *_(workspace_buttons)[10];

static GtkWidget *_(htop);

static GtkWidget *_(weather);
static GtkWidget *_(weather_label);

static GtkWidget *_(language);
static GtkWidget *_(language_label);

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
  _(worspaces) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_widget_add_css_class(_(worspaces), "widget");
  gtk_widget_add_css_class(_(worspaces), "workspaces");
  for (size_t i = 0; i < 10; i++) {
    GtkWidget *button = gtk_button_new();
    char buffer[3];
    sprintf(buffer, "%lu", i + 1);
    GtkWidget *label = gtk_label_new(buffer);
    gtk_button_set_child(GTK_BUTTON(button), label);
    gtk_box_append(GTK_BOX(_(worspaces)), button);
    _(workspace_buttons)[i] = button;
  }
  gtk_box_append(GTK_BOX(left), _(worspaces));

  // htop
  _(htop) = gtk_button_new();
  gtk_widget_add_css_class(_(htop), "widget");
  gtk_widget_add_css_class(_(htop), "terminal");
  gtk_widget_add_css_class(_(htop), "padded");
  gtk_widget_add_css_class(_(htop), "clickable");
  GtkWidget *htop_label = gtk_label_new("Htop");
  gtk_button_set_child(GTK_BUTTON(_(htop)), htop_label);
  gtk_box_append(GTK_BOX(right), _(htop));

  // weather
  _(weather_label) = gtk_label_new("--");
  _(weather) = gtk_button_new();
  gtk_widget_add_css_class(_(weather), "widget");
  gtk_widget_add_css_class(_(weather), "weather");
  gtk_widget_add_css_class(_(weather), "padded");
  gtk_widget_add_css_class(_(weather), "clickable");
  gtk_button_set_child(GTK_BUTTON(_(weather)), _(weather_label));
  gtk_box_append(GTK_BOX(right), _(weather));

  // language
  _(language_label) = gtk_label_new("--");
  _(language) = gtk_center_box_new();
  gtk_widget_add_css_class(_(language), "widget");
  gtk_widget_add_css_class(_(language), "language");
  gtk_widget_add_css_class(_(language), "padded");
  gtk_center_box_set_center_widget(GTK_CENTER_BOX(_(language)),
                                   _(language_label));
  gtk_box_append(GTK_BOX(right), _(language));

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
  GtkWidget *network_wrapper = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_box_append(GTK_BOX(network_wrapper), _(network_label));
  gtk_box_append(GTK_BOX(network_wrapper), _(network_image));
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

static void _(workspace_button_on_click)(GtkButton *, gpointer data) {
  size_t idx = (size_t)data;
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = HyprlandGoToWorkspace, .hyprland_go_to_workspace = {idx}});
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
  case Workspaces: {
    for (size_t idx = 1; idx <= 10; idx++) {
      GtkWidget *button = _(workspace_buttons)[idx - 1];
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
      gtk_label_set_label(GTK_LABEL(_(language_label)), "EN");
    } else if (strcmp(event->language.lang.ptr, "Polish") == 0) {
      gtk_label_set_label(GTK_LABEL(_(language_label)), "PL");
    } else {
      gtk_label_set_label(GTK_LABEL(_(language_label)), "??");
    }
    break;
  }
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
  case WiFiStatus: {
    if (event->wi_fi_status.ssid.ptr == NULL) {
      gtk_widget_set_visible(_(network_image), false);
      gtk_label_set_label(GTK_LABEL(_(network_label)), "Not connected");
    } else {
      gtk_widget_set_visible(_(network_image), true);
      char buffer[100];
      sprintf(buffer, "%s (%d)%% ", event->wi_fi_status.ssid.ptr,
              event->wi_fi_status.strength);
      gtk_label_set_label(GTK_LABEL(_(network_label)), buffer);
    }
    break;
  }
  case Time: {
    gtk_label_set_label(GTK_LABEL(_(time_label)), event->time.time.ptr);
    gtk_widget_set_tooltip_text(_(time_label), event->time.date.ptr);
    break;
  }
  case CurrentWeather: {
    char buffer[100];
    sprintf(buffer, "%.1f℃ %s", event->current_weather.temperature,
            weather_code_to_description(event->current_weather.code));
    gtk_label_set_label(GTK_LABEL(_(weather_label)), buffer);
    break;
  }

  default:
    break;
  }
}

static bool _(bottom_right_point_of)(GtkWidget *widget, graphene_point_t *out) {
  graphene_rect_t bounds;
  if (!gtk_widget_compute_bounds(widget, GTK_WIDGET(_(window)), &bounds)) {
    return false;
  }

  out->x = bounds.origin.x + bounds.size.width;
  out->y = bounds.origin.y + bounds.size.height;

  return true;
}

static void _(htop_btn_on_click)() {
  graphene_point_t bottom_right;
  if (!_(bottom_right_point_of)(_(htop), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the htop widget");
    return;
  }
  int margin_left = bottom_right.x - HTOP.width / 2.0;
  int margin_top = bottom_right.y;
  HTOP.move(margin_left, margin_top);

  HTOP.toggle();
}

static void _(weather_btn_on_click)() {
  graphene_point_t bottom_right;
  if (!_(bottom_right_point_of)(_(weather), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the weather widget");
    return;
  }
  int margin_left = bottom_right.x - WEATHER.width;
  int margin_top = bottom_right.y;
  WEATHER.move(margin_left, margin_top);

  WEATHER.toggle();
}

static void _(network_btn_on_click)() {
  graphene_point_t bottom_right;
  if (!_(bottom_right_point_of)(_(network), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the network widget");
    return;
  }
  int margin_left = bottom_right.x - NETWORK.width;
  int margin_top = bottom_right.y;
  NETWORK.move(margin_left, margin_top);

  NETWORK.toggle();
}

static void _(activate)(GApplication *app) {
  gtk_window_set_application(_(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(_(window));
  gtk_layer_set_layer(_(window), GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(_(window), GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(_(window), "LayerShell/TopBar");

  for (size_t idx = 0; idx < 10; idx++) {
    GtkWidget *button = _(workspace_buttons)[idx];
    g_signal_connect(button, "clicked",
                     G_CALLBACK(_(workspace_button_on_click)), (void *)idx);
  }

  g_signal_connect(_(htop), "clicked", _(htop_btn_on_click), NULL);

  g_signal_connect(_(weather), "clicked", _(weather_btn_on_click), NULL);

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

window_t TOP_BAR = {.init = _(init), .activate = _(activate)};
