#include "ui/include/top_bar/tray_app_icon.h"
#include "ui/include/icons.h"

static GtkWidget *
image_from_pixmap_variant(IO_TrayIcon_IO_PixmapVariant_Body pixmap_variant) {
  GBytes *bytes =
      g_bytes_new(pixmap_variant.bytes.ptr, pixmap_variant.bytes.len);
  GdkPixbuf *pixbuf = gdk_pixbuf_new_from_bytes(
      bytes, GDK_COLORSPACE_RGB, true, 8, pixmap_variant.w, pixmap_variant.h,
      4 * pixmap_variant.w);
  GdkTexture *texture = gdk_texture_new_for_pixbuf(pixbuf);
  return gtk_image_new_from_paintable(GDK_PAINTABLE(texture));
}

GtkWidget *tray_app_icon_new(IO_TrayIcon tray_icon) {
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
