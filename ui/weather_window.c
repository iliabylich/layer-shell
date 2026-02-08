#include "ui/weather_window.h"
#include "ui/assertions.h"
#include "ui/base_window.h"
#include "ui/logger.h"
#include "ui/weather_helper.h"
#include <gtk4-layer-shell.h>

LOGGER("WeatherWindow", 0)

struct _WeatherWindow {
  GtkWidget parent_instance;

  GtkWidget *root;

  GtkWidget *hourly;
  GtkWidget *daily;
};

G_DEFINE_TYPE(WeatherWindow, weather_window, BASE_WINDOW_TYPE)

#define HOURLY_COLS_COUNT 3
#define HOURLY_ROWS_COUNT 10

#define DAILY_COLS_COUNT 4
#define DAILY_ROWS_COUNT 6

static void allocate_grid_size(GtkWidget *grid, size_t cols_count,
                               size_t rows_count) {
  for (size_t col = 0; col < cols_count; col++) {
    gtk_grid_insert_column(GTK_GRID(grid), col);
  }

  for (size_t row = 0; row < rows_count; row++) {
    gtk_grid_insert_row(GTK_GRID(grid), row);
  }
}

static void init_grid_cell(GtkWidget *grid, size_t col, size_t row,
                           GtkWidget *cell) {
  gtk_grid_attach(GTK_GRID(grid), cell, col, row, 1, 1);
}

static GtkWidget *temperature_label_new();
static GtkWidget *temperature_icon_new();

static void weather_window_init(WeatherWindow *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
  gtk_widget_add_css_class(GTK_WIDGET(self), "weather-window");

  base_window_set_toggle_on_escape(BASE_WINDOW(self));

  self->hourly = gtk_grid_new();
  allocate_grid_size(self->hourly, HOURLY_COLS_COUNT, HOURLY_ROWS_COUNT);
  for (size_t row = 0; row < HOURLY_ROWS_COUNT; row++) {
    init_grid_cell(self->hourly, 0, row, gtk_label_new("??"));
    init_grid_cell(self->hourly, 1, row, temperature_label_new());
    init_grid_cell(self->hourly, 2, row, temperature_icon_new());
  }

  self->daily = gtk_grid_new();
  allocate_grid_size(self->daily, DAILY_COLS_COUNT, DAILY_ROWS_COUNT);
  for (size_t row = 0; row < DAILY_ROWS_COUNT; row++) {
    init_grid_cell(self->daily, 0, row, gtk_label_new("??"));
    init_grid_cell(self->daily, 1, row, temperature_label_new());
    init_grid_cell(self->daily, 2, row, temperature_label_new());
    init_grid_cell(self->daily, 3, row, temperature_icon_new());
  }

  GtkWidget *left = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_box_append(GTK_BOX(left), gtk_label_new("Hourly"));
  gtk_box_append(GTK_BOX(left), self->hourly);

  GtkWidget *right = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_box_append(GTK_BOX(right), gtk_label_new("Daily"));
  gtk_box_append(GTK_BOX(right), self->daily);

  self->root = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 50);
  gtk_box_append(GTK_BOX(self->root), left);
  gtk_box_append(GTK_BOX(self->root), right);

  gtk_window_set_child(GTK_WINDOW(self), self->root);
}

static void weather_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(weather_window_parent_class)->dispose(object);
}

static void weather_window_class_init(WeatherWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = weather_window_dispose;
}

GtkWidget *weather_window_new(GtkApplication *app) {
  return g_object_new(weather_window_get_type(), "application", app, NULL);
}

void weather_window_toggle(WeatherWindow *self) {
  base_window_toggle(BASE_WINDOW(self));
}

static GtkWidget *temperature_label_new() { return gtk_label_new("??"); }
static void temperature_label_refresh(GtkWidget *label, float temperature) {
  char buffer[100];
  checked_fmt(buffer, "%5.1f℃", temperature);
  gtk_label_set_label(GTK_LABEL(label), buffer);
}

static GtkWidget *temperature_icon_new() {
  GtkWidget *label = gtk_label_new("");
  gtk_widget_add_css_class(label, "icon");
  return label;
}
static void temperature_icon_refresh(GtkWidget *icon, IO_WeatherCode code) {
  gtk_label_set_label(GTK_LABEL(icon), weather_code_to_icon(code));
  gtk_widget_set_tooltip_text(icon, weather_code_to_description(code));
}

void weather_window_refresh_hourly_forecast(
    WeatherWindow *self, struct IO_FFIArray_WeatherOnHour data) {
  GtkWidget *child;
  for (size_t row = 0; row < data.len && row < HOURLY_ROWS_COUNT; row++) {
    IO_WeatherOnHour weather_on_hour = data.ptr[row];

    child = gtk_grid_get_child_at(GTK_GRID(self->hourly), 0, row);
    gtk_label_set_text(GTK_LABEL(child), weather_on_hour.hour);

    child = gtk_grid_get_child_at(GTK_GRID(self->hourly), 1, row);
    temperature_label_refresh(child, weather_on_hour.temperature);

    child = gtk_grid_get_child_at(GTK_GRID(self->hourly), 2, row);
    temperature_icon_refresh(child, weather_on_hour.code);
  }
}

void weather_window_refresh_daily_forecast(
    WeatherWindow *self, struct IO_FFIArray_WeatherOnDay data) {
  GtkWidget *child;

  for (size_t row = 0; row < data.len && row < DAILY_ROWS_COUNT; row++) {
    IO_WeatherOnDay weather_on_day = data.ptr[row];

    child = gtk_grid_get_child_at(GTK_GRID(self->daily), 0, row);
    gtk_label_set_text(GTK_LABEL(child), weather_on_day.day);

    child = gtk_grid_get_child_at(GTK_GRID(self->daily), 1, row);
    temperature_label_refresh(child, weather_on_day.temperature_min);

    child = gtk_grid_get_child_at(GTK_GRID(self->daily), 2, row);
    temperature_label_refresh(child, weather_on_day.temperature_max);

    child = gtk_grid_get_child_at(GTK_GRID(self->daily), 3, row);
    temperature_icon_refresh(child, weather_on_day.code);
  }
}
