#include "ui/tray_icon.h"
#include "ui/logger.h"
#include "ui/tray_popover.h"

LOGGER("TrayIcon", 2)

enum {
  SIGNAL_TRIGGERED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _TrayIcon {
  GtkWidget parent_instance;

  GtkWidget *root;
  GtkWidget *image;
  GtkWidget *popover;
};

G_DEFINE_TYPE(TrayIcon, tray_icon, GTK_TYPE_WIDGET)

#define EMPTY_ICON_NAME "process-stop"

static void on_click(GtkWidget *, TrayIcon *self) {
  tray_popover_open(TRAY_POPOVER(self->popover));
}

static void on_trigger(GtkWidget *, const char *uuid, TrayIcon *self) {
  g_signal_emit(self, signals[SIGNAL_TRIGGERED], 0, uuid);
}

static void tray_icon_init(TrayIcon *self) {
  LOG("init");

  self->root = gtk_button_new();
  gtk_widget_set_cursor_from_name(self->root, "pointer");
  g_signal_connect(self->root, "clicked", G_CALLBACK(on_click), self);

  self->image = gtk_image_new_from_icon_name(EMPTY_ICON_NAME);
  gtk_button_set_child(GTK_BUTTON(self->root), self->image);

  self->popover = tray_popover_new();
  g_signal_connect(self->popover, "triggered", G_CALLBACK(on_trigger), self);

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
  gtk_widget_set_parent(self->popover, GTK_WIDGET(self->root));
}

static void tray_icon_dispose(GObject *object) {
  LOG("dispose");

  TrayIcon *self = TRAY_ICON(object);
  g_clear_pointer(&self->popover, gtk_widget_unparent);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(tray_icon_parent_class)->dispose(object);
}

static void tray_icon_class_init(TrayIconClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = tray_icon_dispose;
  signals[SIGNAL_TRIGGERED] = g_signal_new_class_handler(
      "triggered", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *tray_icon_new(IO_TrayIcon icon, IO_CArray_TrayItem items) {
  GtkWidget *self = g_object_new(tray_icon_get_type(), NULL);
  tray_icon_update_icon(TRAY_ICON(self), icon);
  tray_icon_update_menu(TRAY_ICON(self), items);
  return self;
}

void tray_icon_update_icon(TrayIcon *self, IO_TrayIcon icon) {
  GtkImage *image = GTK_IMAGE(self->image);

  switch (icon.tag) {
  case IO_TrayIcon_Path: {
    gtk_image_set_from_file(image, icon.path.path);
    break;
  }
  case IO_TrayIcon_Name: {
    gtk_image_set_from_icon_name(image, icon.name.name);
    break;
  }
  case IO_TrayIcon_Pixmap: {
    uint8_t *data = icon.pixmap.bytes.ptr;
    size_t size = icon.pixmap.bytes.len;
    uint32_t w = icon.pixmap.width;
    uint32_t h = icon.pixmap.height;

    GBytes *bytes = g_bytes_new(data, size);
    GdkPixbuf *pixbuf = gdk_pixbuf_new_from_bytes(bytes, GDK_COLORSPACE_RGB,
                                                  true, 8, w, h, 4 * w);
    GdkTexture *texture = gdk_texture_new_for_pixbuf(pixbuf);
    gtk_image_set_from_paintable(image, GDK_PAINTABLE(texture));

    g_bytes_unref(bytes);
    g_object_unref(pixbuf);
    g_object_unref(texture);
    break;
  }
  case IO_TrayIcon_Unset: {
    gtk_image_set_from_icon_name(image, EMPTY_ICON_NAME);
    break;
  }
  }
}

void tray_icon_update_menu(TrayIcon *self, IO_CArray_TrayItem items) {
  tray_popover_update(TRAY_POPOVER(self->popover), items);
}
