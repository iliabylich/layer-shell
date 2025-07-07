#include "ui/workspaces_button.h"
#include "ui/logger.h"

LOGGER("WorkspacesButton", 2)

enum {
  PROP_NUM = 1,
  PROP_ACTIVE,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

enum {
  SIGNAL_TRIGGERED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _WorkspacesButton {
  GtkWidget parent_instance;

  GtkWidget *root;
  guint num;
};

G_DEFINE_TYPE(WorkspacesButton, workspaces_button, GTK_TYPE_WIDGET)

static void on_clicked(GtkWidget *, WorkspacesButton *self) {
  g_signal_emit(self, signals[SIGNAL_TRIGGERED], 0, self->num);
}

static void workspaces_button_init(WorkspacesButton *self) {
  LOG("init");

  self->root = gtk_button_new_with_label("?");
  g_signal_connect(self->root, "clicked", G_CALLBACK(on_clicked), self);
  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void workspaces_button_dispose(GObject *object) {
  LOG("dispose");

  WorkspacesButton *self = WORKSPACES_BUTTON(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(workspaces_button_parent_class)->dispose(object);
}

static void workspaces_button_set_property(GObject *object, guint property_id,
                                           const GValue *value,
                                           GParamSpec *pspec) {
  WorkspacesButton *self = WORKSPACES_BUTTON(object);

  switch (property_id) {
  case PROP_NUM:
    self->num = g_value_get_uint(value);
    char label[5];
    sprintf(label, "%u", self->num);
    gtk_button_set_label(GTK_BUTTON(self->root), label);
    break;

  case PROP_ACTIVE:
    bool active = g_value_get_boolean(value);
    if (active) {
      gtk_widget_add_css_class(self->root, "active");
    } else {
      gtk_widget_remove_css_class(self->root, "active");
    }
    break;

  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void workspaces_button_class_init(WorkspacesButtonClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = workspaces_button_dispose;
  object_class->set_property = workspaces_button_set_property;

  signals[SIGNAL_TRIGGERED] = g_signal_new_class_handler(
      "triggered", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_UINT);

  properties[PROP_NUM] =
      g_param_spec_uint("num", NULL, NULL, 0, 100, 0, G_PARAM_WRITABLE);
  properties[PROP_ACTIVE] =
      g_param_spec_boolean("active", NULL, NULL, false, G_PARAM_WRITABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *workspaces_button_new(guint num) {
  return g_object_new(workspaces_button_get_type(), "num", num, NULL);
}
