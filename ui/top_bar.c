#include "ui/top_bar.h"
#include "ui/cpu_item.h"
#include "ui/io_model.h"
#include "ui/logger.h"
#include "ui/tray_app_item.h"
#include "ui/workspace_item.h"
#include <gtk4-layer-shell.h>

LOGGER("TopBar", 0)

static char *format_workspace_num(GObject *, guint num) {
  return g_strdup_printf("%u", num);
}

static GVariant *workspace_action_target(GObject *, guint num) {
  return g_variant_new_uint32(num);
}

static const char *CPU_INDICATORS[] = {
    "<span color='#FFFFFF'>\xe2\x96\x81</span>",
    "<span color='#FFD5D5'>\xe2\x96\x82</span>",
    "<span color='#FFAAAA'>\xe2\x96\x83</span>",
    "<span color='#FF8080'>\xe2\x96\x84</span>",
    "<span color='#FF5555'>\xe2\x96\x85</span>",
    "<span color='#FF2B2B'>\xe2\x96\x86</span>",
    "<span color='#FF0000'>\xe2\x96\x87</span>",
    "<span color='#E60000'>\xe2\x96\x88</span>",
};
static const size_t CPU_INDICATORS_COUNT =
    sizeof(CPU_INDICATORS) / sizeof(const char *);

static char *format_cpu_load(GObject *, guint load) {
  size_t idx = (size_t)(load / 100.0 * CPU_INDICATORS_COUNT);
  if (idx >= CPU_INDICATORS_COUNT)
    idx = CPU_INDICATORS_COUNT - 1;
  return g_strdup(CPU_INDICATORS[idx]);
}

static char *format_memory_label(GObject *, double used, double total) {
  if (total <= 0.0)
    return g_strdup("RAM --");
  return g_strdup_printf("RAM %.1fG/%.1fG", used, total);
}

static char *format_clock_label(GObject *, int64_t unix_seconds) {
  if (unix_seconds <= 0)
    return g_strdup("--");

  GDateTime *dt = g_date_time_new_from_unix_local(unix_seconds);
  if (!dt)
    return g_strdup("--");

  char *formatted = g_date_time_format(dt, "%H:%M:%S | %b %e | %a");
  g_date_time_unref(dt);
  return formatted ? formatted : g_strdup("--");
}

struct _TopBar {
  GtkWidget parent_instance;

  GtkWidget *change_theme;
  GtkWidget *weather;
  GtkWidget *terminal;
  GtkWidget *memory;
  GtkWidget *bluetooth;
  GtkWidget *power;

  IOModel *model;
};

G_DEFINE_TYPE(TopBar, top_bar, GTK_TYPE_WINDOW)

enum {
  PROP_MODEL = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

enum {
  SIGNAL_WORKSPACE_SWITCHED = 0,
  SIGNAL_CHANGE_THEME_CLICKED,
  SIGNAL_TRAY_TRIGGERED,
  SIGNAL_WEATHER_CLICKED,
  SIGNAL_TERMINAL_CLICKED,
  SIGNAL_MEMORY_CLICKED,
  SIGNAL_NETWORK_SETTINGS_CLICKED,
  SIGNAL_NETWORK_PING_CLICKED,
  SIGNAL_BLUETOOTH_CLICKED,
  SIGNAL_POWER_CLICKED,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void on_tray_items_changed(GListModel *, guint, guint, guint, gpointer);

static void top_bar_get_property(GObject *object, guint property_id,
                                 GValue *value, GParamSpec *pspec) {
  TopBar *self = TOP_BAR(object);
  switch (property_id) {
  case PROP_MODEL:
    g_value_set_object(value, self->model);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void top_bar_set_property(GObject *object, guint property_id,
                                 const GValue *value, GParamSpec *pspec) {
  TopBar *self = TOP_BAR(object);
  switch (property_id) {
  case PROP_MODEL: {
    g_set_object(&self->model, g_value_get_object(value));
    GListModel *tray_apps = NULL;
    g_object_get(self->model, "tray-apps", &tray_apps, NULL);
    g_signal_connect(tray_apps, "items-changed",
                     G_CALLBACK(on_tray_items_changed), self);
    g_object_unref(tray_apps);
    break;
  }
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void on_ws_switch(GSimpleAction *, GVariant *parameter, gpointer data) {
  TopBar *self = TOP_BAR(data);
  guint num = g_variant_get_uint32(parameter);
  g_signal_emit(self, signals[SIGNAL_WORKSPACE_SWITCHED], 0, num);
}

#define FORWARD_CLICKED(name, sig)                                             \
  static void name(GtkButton *, gpointer data) {                               \
    g_signal_emit(data, signals[sig], 0);                                      \
  }

FORWARD_CLICKED(on_change_theme_clicked, SIGNAL_CHANGE_THEME_CLICKED)
FORWARD_CLICKED(on_weather_clicked, SIGNAL_WEATHER_CLICKED)
FORWARD_CLICKED(on_terminal_clicked, SIGNAL_TERMINAL_CLICKED)
FORWARD_CLICKED(on_memory_clicked, SIGNAL_MEMORY_CLICKED)
FORWARD_CLICKED(on_bluetooth_clicked, SIGNAL_BLUETOOTH_CLICKED)
FORWARD_CLICKED(on_power_clicked, SIGNAL_POWER_CLICKED)
#undef FORWARD_CLICKED

static void on_tray_triggered(TrayAppItem *, const char *uuid, gpointer data) {
  g_signal_emit(data, signals[SIGNAL_TRAY_TRIGGERED], 0, uuid);
}

static void on_tray_items_changed(GListModel *list, guint position, guint,
                                  guint added, gpointer data) {
  for (guint i = position; i < position + added; i++) {
    TrayAppItem *item = g_list_model_get_item(list, i);
    g_signal_connect(item, "triggered", G_CALLBACK(on_tray_triggered), data);
    g_object_unref(item);
  }
}

static void on_network_settings(GSimpleAction *, GVariant *, gpointer data) {
  g_signal_emit(data, signals[SIGNAL_NETWORK_SETTINGS_CLICKED], 0);
}

static void on_network_ping(GSimpleAction *, GVariant *, gpointer data) {
  g_signal_emit(data, signals[SIGNAL_NETWORK_PING_CLICKED], 0);
}

static void top_bar_init(TopBar *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/TopBar");
  gtk_layer_auto_exclusive_zone_enable(GTK_WINDOW(self));

  gtk_widget_init_template(GTK_WIDGET(self));

  GSimpleAction *ws_action =
      g_simple_action_new("switch", G_VARIANT_TYPE_UINT32);
  g_signal_connect(ws_action, "activate", G_CALLBACK(on_ws_switch), self);
  GSimpleActionGroup *ws_group = g_simple_action_group_new();
  g_action_map_add_action(G_ACTION_MAP(ws_group), G_ACTION(ws_action));
  gtk_widget_insert_action_group(GTK_WIDGET(self), "ws",
                                 G_ACTION_GROUP(ws_group));

#define CONNECT(widget, signal, callback)                                      \
  g_signal_connect(widget, signal, G_CALLBACK(callback), self)

  CONNECT(self->change_theme, "clicked", on_change_theme_clicked);
  CONNECT(self->weather, "clicked", on_weather_clicked);
  CONNECT(self->terminal, "clicked", on_terminal_clicked);
  CONNECT(self->memory, "clicked", on_memory_clicked);
  CONNECT(self->bluetooth, "clicked", on_bluetooth_clicked);
  CONNECT(self->power, "clicked", on_power_clicked);

#undef CONNECT

  GSimpleActionGroup *net_group = g_simple_action_group_new();
  GSimpleAction *net_settings = g_simple_action_new("settings", NULL);
  g_signal_connect(net_settings, "activate", G_CALLBACK(on_network_settings),
                   self);
  g_action_map_add_action(G_ACTION_MAP(net_group), G_ACTION(net_settings));
  GSimpleAction *net_ping = g_simple_action_new("ping", NULL);
  g_signal_connect(net_ping, "activate", G_CALLBACK(on_network_ping), self);
  g_action_map_add_action(G_ACTION_MAP(net_group), G_ACTION(net_ping));
  gtk_widget_insert_action_group(GTK_WIDGET(self), "network",
                                 G_ACTION_GROUP(net_group));
}

static void top_bar_dispose(GObject *object) {
  LOG("dispose");
  TopBar *self = TOP_BAR(object);
  g_clear_object(&self->model);
  G_OBJECT_CLASS(top_bar_parent_class)->dispose(object);
}

static void top_bar_class_init(TopBarClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = top_bar_dispose;
  object_class->get_property = top_bar_get_property;
  object_class->set_property = top_bar_set_property;

  properties[PROP_MODEL] = g_param_spec_object(
      "model", NULL, NULL, io_model_get_type(), G_PARAM_READWRITE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  signals[SIGNAL_WORKSPACE_SWITCHED] = g_signal_new(
      "workspace-switched", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_UINT);

#define SIGNAL_CLICKED(id, name)                                               \
  signals[id] =                                                                \
      g_signal_new(name, G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0, NULL, \
                   NULL, NULL, G_TYPE_NONE, 0)

  SIGNAL_CLICKED(SIGNAL_CHANGE_THEME_CLICKED, "change-theme-clicked");
  SIGNAL_CLICKED(SIGNAL_WEATHER_CLICKED, "weather-clicked");
  SIGNAL_CLICKED(SIGNAL_TERMINAL_CLICKED, "terminal-clicked");
  SIGNAL_CLICKED(SIGNAL_MEMORY_CLICKED, "memory-clicked");
  SIGNAL_CLICKED(SIGNAL_BLUETOOTH_CLICKED, "bluetooth-clicked");
  SIGNAL_CLICKED(SIGNAL_POWER_CLICKED, "power-clicked");
  SIGNAL_CLICKED(SIGNAL_NETWORK_SETTINGS_CLICKED, "network-settings-clicked");
  SIGNAL_CLICKED(SIGNAL_NETWORK_PING_CLICKED, "network-ping-clicked");

#undef SIGNAL_CLICKED

  signals[SIGNAL_TRAY_TRIGGERED] = g_signal_new(
      "tray-triggered", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0, NULL,
      NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);

  g_type_ensure(tray_app_item_get_type());
  g_type_ensure(workspace_item_get_type());
  g_type_ensure(cpu_item_get_type());

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/top_bar.ui");
  gtk_widget_class_bind_template_callback(widget_class, format_workspace_num);
  gtk_widget_class_bind_template_callback(widget_class,
                                          workspace_action_target);
  gtk_widget_class_bind_template_callback(widget_class, format_cpu_load);
  gtk_widget_class_bind_template_callback(widget_class, format_memory_label);
  gtk_widget_class_bind_template_callback(widget_class, format_clock_label);
  gtk_widget_class_bind_template_child(widget_class, TopBar, change_theme);
  gtk_widget_class_bind_template_child(widget_class, TopBar, weather);
  gtk_widget_class_bind_template_child(widget_class, TopBar, terminal);
  gtk_widget_class_bind_template_child(widget_class, TopBar, bluetooth);
  gtk_widget_class_bind_template_child(widget_class, TopBar, memory);
  gtk_widget_class_bind_template_child(widget_class, TopBar, power);
}

GtkWidget *top_bar_new(GtkApplication *app) {
  return g_object_new(top_bar_get_type(), "application", app, NULL);
}

void top_bar_set_model(TopBar *self, IOModel *model) {
  g_object_set(self, "model", model, NULL);
}

void top_bar_set_terminal_label(TopBar *self, const char *label) {
  gtk_button_set_label(GTK_BUTTON(self->terminal), label);
}
