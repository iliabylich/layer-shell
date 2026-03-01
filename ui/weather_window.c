#include "ui/weather_window.h"
#include "ui/assertions.h"
#include "ui/logger.h"
#include "ui/weather_helper.h"

LOGGER("WeatherWindow", 0)

struct _WeatherWindow {
  GtkWidget parent_instance;

  GtkGrid *hourly;
  GtkGrid *daily;
};

G_DEFINE_TYPE(WeatherWindow, weather_window, BASE_WINDOW_TYPE)

#define HOURLY_COLS_COUNT 3
#define HOURLY_ROWS_COUNT 10

#define DAILY_COLS_COUNT 4
#define DAILY_ROWS_COUNT 6

static GtkWidget *temperature_label_new();
static GtkWidget *temperature_icon_new();

static void weather_window_init(WeatherWindow *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));

  for (size_t row = 0; row < HOURLY_ROWS_COUNT; row++) {
    gtk_grid_attach(self->hourly, gtk_label_new("??"), 0, row, 1, 1);
    gtk_grid_attach(self->hourly, temperature_label_new(), 1, row, 1, 1);
    gtk_grid_attach(self->hourly, temperature_icon_new(), 2, row, 1, 1);
  }

  for (size_t row = 0; row < DAILY_ROWS_COUNT; row++) {
    gtk_grid_attach(self->daily, gtk_label_new("??"), 0, row, 1, 1);
    gtk_grid_attach(self->daily, temperature_label_new(), 1, row, 1, 1);
    gtk_grid_attach(self->daily, temperature_label_new(), 2, row, 1, 1);
    gtk_grid_attach(self->daily, temperature_icon_new(), 3, row, 1, 1);
  }
}

static void weather_window_dispose(GObject *object) {
  LOG("dispose");
  gtk_widget_dispose_template(GTK_WIDGET(object), weather_window_get_type());
  G_OBJECT_CLASS(weather_window_parent_class)->dispose(object);
}

static void weather_window_class_init(WeatherWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = weather_window_dispose;

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/weather_window.ui");
  gtk_widget_class_bind_template_child(widget_class, WeatherWindow, hourly);
  gtk_widget_class_bind_template_child(widget_class, WeatherWindow, daily);
}

GtkWidget *weather_window_new(GtkApplication *app) {
  return g_object_new(weather_window_get_type(), "application", app, NULL);
}

static GtkWidget *temperature_label_new() { return gtk_label_new("??"); }
static void temperature_label_refresh(GtkWidget *label, float temperature) {
  char buffer[100];
  checked_fmt(buffer, "%5.1f℃", temperature);
  gtk_label_set_label(GTK_LABEL(label), buffer);
}

static GtkWidget *temperature_icon_new() {
  GtkWidget *label = gtk_label_new("");
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

    child = gtk_grid_get_child_at(self->hourly, 0, row);
    gtk_label_set_text(GTK_LABEL(child), weather_on_hour.hour);

    child = gtk_grid_get_child_at(self->hourly, 1, row);
    temperature_label_refresh(child, weather_on_hour.temperature);

    child = gtk_grid_get_child_at(self->hourly, 2, row);
    temperature_icon_refresh(child, weather_on_hour.code);
  }
}

void weather_window_refresh_daily_forecast(
    WeatherWindow *self, struct IO_FFIArray_WeatherOnDay data) {
  GtkWidget *child;

  for (size_t row = 0; row < data.len && row < DAILY_ROWS_COUNT; row++) {
    IO_WeatherOnDay weather_on_day = data.ptr[row];

    child = gtk_grid_get_child_at(self->daily, 0, row);
    gtk_label_set_text(GTK_LABEL(child), weather_on_day.day);

    child = gtk_grid_get_child_at(self->daily, 1, row);
    temperature_label_refresh(child, weather_on_day.temperature_min);

    child = gtk_grid_get_child_at(self->daily, 2, row);
    temperature_label_refresh(child, weather_on_day.temperature_max);

    child = gtk_grid_get_child_at(self->daily, 3, row);
    temperature_icon_refresh(child, weather_on_day.code);
  }
}
