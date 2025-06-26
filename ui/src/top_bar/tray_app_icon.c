#include "ui/include/top_bar/tray_app_icon.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_app_icon_popover.h"

#define POPOVER "popover"
static void tray_icon_set_popover(GtkWidget *self, GtkWidget *popover) {
  g_object_set_data(G_OBJECT(self), POPOVER, popover);
}
static GtkWidget *tray_icon_get_popover(GtkWidget *self) {
  return g_object_get_data(G_OBJECT(self), POPOVER);
}

static GtkWidget *
image_from_pixmap_variant(IO_TrayIcon_IO_PixmapVariant_Body pixmap_variant) {
  uint8_t *data = pixmap_variant.bytes.ptr;
  size_t size = pixmap_variant.bytes.len;
  uint32_t w = pixmap_variant.w;
  uint32_t h = pixmap_variant.h;

  GBytes *bytes = g_bytes_new(data, size);
  GdkPixbuf *pixbuf = gdk_pixbuf_new_from_bytes(bytes, GDK_COLORSPACE_RGB, true,
                                                8, w, h, 4 * w);
  GdkTexture *texture = gdk_texture_new_for_pixbuf(pixbuf);
  return gtk_image_new_from_paintable(GDK_PAINTABLE(texture));
}

static GtkWidget *icon_new(IO_TrayIcon tray_icon) {
  switch (tray_icon.tag) {
  case IO_TrayIcon_Path: {
    return gtk_image_new_from_file(tray_icon.path.path);
  }
  case IO_TrayIcon_Name: {
    return gtk_image_new_from_icon_name(tray_icon.name.name);
  }
  case IO_TrayIcon_PixmapVariant: {
    return image_from_pixmap_variant(tray_icon.pixmap_variant);
  }
  case IO_TrayIcon_Unset: {
    return gtk_image_new_from_icon_name("process-stop");
  }
  default: {
    fprintf(stderr, "Unknown tray app icon tag\n");
    return NULL;
  }
  }
}

static void on_click(GtkGestureClick *, gint, gdouble, gdouble,
                     GtkWidget *popover) {
  gtk_popover_popup(GTK_POPOVER(popover));
}

GtkWidget *tray_app_icon_new(IO_TrayIcon icon, IO_TrayItem item,
                             tray_triggered_f cb) {
  GtkWidget *self = icon_new(icon);

  GtkWidget *popover = tray_app_icon_popover_new(item, cb);
  gtk_widget_set_parent(popover, self);
  tray_icon_set_popover(self, popover);

  GtkGesture *gesture = gtk_gesture_click_new();
  g_signal_connect(gesture, "pressed", G_CALLBACK(on_click), popover);
  gtk_widget_add_controller(self, GTK_EVENT_CONTROLLER(gesture));

  return self;
}

void tray_app_icon_cleanup(GtkWidget *self) {
  GtkWidget *popover = tray_icon_get_popover(self);
  gtk_widget_unparent(popover);
  gtk_widget_unparent(GTK_WIDGET(self));
}
