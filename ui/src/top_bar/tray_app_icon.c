#include "ui/include/top_bar/tray_app_icon.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_app_icon_context.h"
#include "ui/include/top_bar/tray_app_icon_popover.h"

typedef struct {
  GtkWidget *icon;
  GtkWidget *popover;
  GList *context_pool;
  tray_triggered_f callback;
} data_t;
#define DATA_KEY "data"

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
                     GtkWidget *popover_menu) {
  gtk_popover_popup(GTK_POPOVER(popover_menu));
}

GtkWidget *tray_app_icon_new(IO_TrayApp tray_app, GtkWidget *tray) {
  GtkWidget *self = icon_new(tray_app.icon);

  data_t *data = malloc(sizeof(data_t));
  data->context_pool = NULL;
  data->icon = self;
  data->popover =
      tray_app_icon_popover_new(tray_app.root_item, tray, &data->context_pool);
  gtk_widget_set_parent(data->popover, self);

  GtkGesture *gesture = gtk_gesture_click_new();
  g_signal_connect(gesture, "pressed", G_CALLBACK(on_click), data->popover);
  gtk_widget_add_controller(self, GTK_EVENT_CONTROLLER(gesture));

  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  return self;
}

void tray_app_icon_cleanup(GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  g_list_free_full(data->context_pool,
                   (GDestroyNotify)tray_app_icon_context_free);
  gtk_widget_unparent(data->popover);
  data->popover = NULL;
  gtk_widget_unparent(GTK_WIDGET(self));
}
