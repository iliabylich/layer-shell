#include "ui/base_window.h"
#include <vte/vte.h>

struct _BaseWindow {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(BaseWindow, base_window, GTK_TYPE_WINDOW)

static void base_window_init(BaseWindow *) {}

static void base_window_dispose(GObject *object) {
  G_OBJECT_CLASS(base_window_parent_class)->dispose(object);
}

static void base_window_class_init(BaseWindowClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = base_window_dispose;
}

void base_window_toggle(BaseWindow *self) {
  gtk_widget_set_visible(GTK_WIDGET(self),
                         !gtk_widget_get_visible(GTK_WIDGET(self)));
}

static bool on_key_pressed(GtkEventControllerKey *, guint keyval, guint,
                           GdkModifierType, BaseWindow *window) {
  const char *keyname = gdk_keyval_name(keyval);
  if (strcmp(keyname, "Escape") == 0) {
    base_window_toggle(window);
    return true;
  } else {
    return false;
  }
}

void base_window_set_toggle_on_escape(BaseWindow *self) {
  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key_pressed", G_CALLBACK(on_key_pressed), self);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(self), ctrl);
}

void base_window_vte(BaseWindow *self, char **command) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "terminal-window");

  GtkWidget *terminal = vte_terminal_new();
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT,
                           getenv("HOME"), command, NULL, G_SPAWN_DEFAULT, NULL,
                           NULL, NULL, -1, NULL, NULL, NULL);
  gtk_window_set_child(GTK_WINDOW(self), terminal);
}
