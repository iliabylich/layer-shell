#include "ui/tracker.h"
#include "bindings.h"
#include "gtk/gtk.h"
#include "ui/logger.h"

LOGGER("Tracker", 1)

enum {
  SIGNAL_CLICKED = 0,
  SIGNAL_RIGHT_CLICKED,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _Tracker {
  GtkWidget parent_instance;

  GtkWidget *root;
};

G_DEFINE_TYPE(Tracker, tracker, GTK_TYPE_WIDGET)

static void on_click(GtkWidget *, Tracker *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED], 0);
}

static void on_right_click(GtkGestureClick *, gint, gdouble, gdouble,
                           Tracker *self) {
  g_signal_emit(self, signals[SIGNAL_RIGHT_CLICKED], 0);
}

static const char *ICON_RUNNING = "󰔟";
static const char *ICON_PAUSED = "󱦠";

static void tracker_init(Tracker *self) {
  LOG("init");

  self->root = gtk_button_new_with_label(ICON_PAUSED);
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "tracker");
  gtk_widget_add_css_class(self->root, "padded");
  gtk_widget_add_css_class(self->root, "clickable");
  gtk_widget_set_cursor_from_name(self->root, "pointer");
  g_signal_connect(self->root, "clicked", G_CALLBACK(on_click), self);

  GtkGesture *gesture = gtk_gesture_click_new();
  gtk_gesture_single_set_button(GTK_GESTURE_SINGLE(gesture), 3);
  g_signal_connect(gesture, "pressed", G_CALLBACK(on_right_click), self);
  gtk_widget_add_controller(GTK_WIDGET(self), GTK_EVENT_CONTROLLER(gesture));

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void tracker_dispose(GObject *object) {
  LOG("dispose");

  Tracker *self = TRACKER(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(tracker_parent_class)->dispose(object);
}

static void tracker_class_init(TrackerClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = tracker_dispose;
  signals[SIGNAL_CLICKED] = g_signal_new_class_handler(
      "clicked", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_RIGHT_CLICKED] = g_signal_new_class_handler(
      "right-clicked", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *tracker_new(void) { return g_object_new(tracker_get_type(), NULL); }

void tracker_refresh(Tracker *self, IO_TrackerUpdatedEvent event) {
  const char *text;
  if (event.view.running) {
    text = ICON_RUNNING;
  } else {
    text = ICON_PAUSED;
  };
  gtk_button_set_label(GTK_BUTTON(self->root), text);
}
