#include "ui/sound_window.h"
#include "ui/base_window.h"
#include "ui/logger.h"
#include <gtk4-layer-shell.h>

LOGGER("SoundWindow", 0)

struct _SoundWindow {
  GtkWidget parent_instance;

  GtkWidget *root;
  GtkWidget *icon;
  GtkWidget *scale;

  uint32_t volume;
  bool muted;

  guint timer;

  bool ready_to_show;
};

G_DEFINE_TYPE(SoundWindow, sound_window, BASE_WINDOW_TYPE)

static void sound_window_init(SoundWindow *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Sound");
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, 100);
  gtk_widget_add_css_class(GTK_WIDGET(self), "sound-window");
  gtk_widget_add_css_class(GTK_WIDGET(self), "notification-window");

  self->root = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 20);
  gtk_widget_add_css_class(self->root, "wrapper");

  self->icon = gtk_label_new("?");
  gtk_box_append(GTK_BOX(self->root), self->icon);

  self->scale = gtk_scale_new_with_range(GTK_ORIENTATION_HORIZONTAL, 0, 100, 1);
  gtk_box_append(GTK_BOX(self->root), self->scale);
  g_object_set(G_OBJECT(self->scale), "width-request", 300, NULL);

  gtk_window_set_child(GTK_WINDOW(self), self->root);

  self->ready_to_show = false;
}

static void sound_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(sound_window_parent_class)->dispose(object);
}

static void sound_window_class_init(SoundWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = sound_window_dispose;
}

GtkWidget *sound_window_new(GtkApplication *app) {
  return g_object_new(sound_window_get_type(), "application", app, NULL);
}

static const char *volume_to_icon(uint32_t volume, bool muted) {
  if (volume == 0 || muted) {
    return "󰝟";
  } else if (volume <= 33) {
    return "󰕿";
  } else if (volume <= 66) {
    return "󰖀";
  } else {
    return "󰕾";
  }
}

static void redraw(SoundWindow *self) {
  gtk_label_set_label(GTK_LABEL(self->icon),
                      volume_to_icon(self->volume, self->muted));
  uint32_t volume_to_show = self->volume;
  if (volume_to_show == 99) {
    volume_to_show = 100;
  }
  gtk_range_set_value(GTK_RANGE(self->scale), volume_to_show);
}

static void hide(gpointer data) {
  SoundWindow *self = data;
  gtk_widget_set_visible(GTK_WIDGET(self), false);
  self->timer = 0;
}

static void show(SoundWindow *self) {
  if (self->ready_to_show) {
    gtk_widget_set_visible(GTK_WIDGET(self), true);

    if (self->timer) {
      g_assert(g_source_remove(self->timer));
    }
    self->timer = g_timeout_add_once(1000, hide, self);
  }
}

void sound_window_set_initial_sound(SoundWindow *self, uint32_t volume,
                                    bool muted) {
  self->volume = volume;
  self->muted = muted;
  self->ready_to_show = true;
}

void sound_window_refresh_volume(SoundWindow *self, uint32_t volume) {
  self->volume = volume;
  redraw(self);
  show(self);
}

void sound_window_refresh_mute(SoundWindow *self, bool muted) {
  self->muted = muted;
  redraw(self);
  show(self);
}
