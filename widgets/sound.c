#include "sound.h"
#include "bindings.h"
#include <gtk/gtk.h>

#define _(name) sound_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(image);
static GtkWidget *_(scale);

static GtkWidget *_(init)(void) {
  _(widget) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "sound");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_set_name(_(widget), "Sound");

  _(image) = gtk_image_new();
  gtk_image_set_from_icon_name(GTK_IMAGE(_(image)), "dialog-question");
  gtk_box_append(GTK_BOX(_(widget)), _(image));

  _(scale) = gtk_scale_new(GTK_ORIENTATION_HORIZONTAL,
                           gtk_adjustment_new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0));
  gtk_widget_add_css_class(_(scale), "sound-slider");

  gtk_box_append(GTK_BOX(_(widget)), _(scale));

  return _(widget);
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_Volume: {
    float volume = event->volume.volume;
    gtk_range_set_value(GTK_RANGE(_(scale)), volume);
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
    gtk_image_set_from_icon_name(GTK_IMAGE(_(image)), icon);
    break;
  }
  default: {
    break;
  }
  }
}

static void _(scale_on_change)(void) {
  GtkAdjustment *adj = gtk_range_get_adjustment(GTK_RANGE(_(scale)));
  double value = CLAMP(gtk_adjustment_get_value(adj), 0.0, 1.0);
  layer_shell_io_publish((IO_Command){.tag = IO_Command_SetVolume,
                                      .set_volume = {.volume = value}});
}

static void _(activate)(void) {
  GtkEventController *sound_ctrl =
      GTK_EVENT_CONTROLLER(gtk_gesture_click_new());
  gtk_event_controller_set_propagation_phase(sound_ctrl, GTK_PHASE_CAPTURE);
  g_signal_connect(sound_ctrl, "released", _(scale_on_change), NULL);
  gtk_widget_add_controller(_(widget), sound_ctrl);

  layer_shell_io_subscribe(_(on_io_event));
}

widget_t SOUND_WIDGET = {.init = _(init), .activate = _(activate)};
