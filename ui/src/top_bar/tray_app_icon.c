#include "ui/include/top_bar/tray_app_icon.h"
#include "ui/include/icons.h"
#include "ui/include/top_bar/tray_app_icon_popover.h"

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
    return gtk_image_new_from_gicon(get_question_mark_icon());
  }
  default: {
    fprintf(stderr, "Unknown tray app icon tag\n");
    return NULL;
  }
  }
}

static void on_click(GtkGestureClick *, gint, gdouble, gdouble,
                     GtkWidget *popover_menu) {
  gtk_popover_popup(GTK_POPOVER(popover_menu));
}

GtkWidget *tray_app_icon_new(IO_TrayApp tray_app, Tray *tray) {
  GtkWidget *icon = icon_new(tray_app.icon);
  GtkWidget *popover_menu = tray_app_icon_popover_new(tray_app.root_item, tray);
  gtk_widget_set_parent(popover_menu, icon);

  GtkGesture *gesture = gtk_gesture_click_new();
  g_signal_connect(gesture, "pressed", G_CALLBACK(on_click), popover_menu);
  gtk_widget_add_controller(icon, GTK_EVENT_CONTROLLER(gesture));

  return icon;
}
