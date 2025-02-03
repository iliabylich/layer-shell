#include "tray.h"
#include "bindings.h"
#include "gdk/gdk.h"
#include "gio/gio.h"
#include "gio/gmenu.h"
#include "glib-object.h"
#include "glib.h"
#include "glibconfig.h"
#include "utils/icons.h"
#include <gtk/gtk.h>
#include <string.h>

#define _(name) tray_widget_ns_##name

static GtkWidget *_(widget);
typedef struct {
  GtkWidget *icon;
  GtkWidget *menu;
} icon_with_menu_t;
#define ICONS_COUNT 10
static icon_with_menu_t _(icons)[ICONS_COUNT];

void on_tray_clicked(GSimpleAction *, GVariant *parameter, gpointer) {
  const char *uuid = g_variant_get_string(parameter, NULL);
  char *owned_uuid = malloc(strlen(uuid) + 1);
  strcpy(owned_uuid, uuid);
  layer_shell_io_publish((IO_Command){.tag = IO_Command_TriggerTray,
                                      .trigger_tray = {.uuid = owned_uuid}});
}

static GtkWidget *_(init)(void) {
  _(widget) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 10);
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "tray");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_set_name(_(widget), "Tray");

  GSimpleActionGroup *action_group = g_simple_action_group_new();
  GSimpleAction *action =
      g_simple_action_new("clicked", g_variant_type_new("s"));
  g_signal_connect(action, "activate", G_CALLBACK(on_tray_clicked), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
  gtk_widget_insert_action_group(_(widget), "tray",
                                 G_ACTION_GROUP(action_group));

  return _(widget);
}

static void _(open_popover)(GtkGestureClick *, int, double x, double y,
                            icon_with_menu_t *icon_with_menu) {
  GdkRectangle rect = {.x = x, .y = y, .height = 1, .width = 1};
  gtk_popover_set_pointing_to(GTK_POPOVER(icon_with_menu->menu), &rect);
  gtk_popover_popup(GTK_POPOVER(icon_with_menu->menu));
}

static void _(set_tray_item)(icon_with_menu_t *icon_with_menu, IO_TrayApp app) {
  switch (app.icon.tag) {
  case IO_TrayIcon_Path: {
    icon_with_menu->icon = gtk_image_new_from_file(app.icon.path.path);
    break;
  }
  case IO_TrayIcon_Name: {
    icon_with_menu->icon = gtk_image_new_from_icon_name(app.icon.name.name);
    break;
  }
  case IO_TrayIcon_PixmapVariant: {
    GBytes *bytes = g_bytes_new(app.icon.pixmap_variant.bytes.ptr,
                                app.icon.pixmap_variant.bytes.len);
    GdkPixbuf *pixbuf = gdk_pixbuf_new_from_bytes(
        bytes, GDK_COLORSPACE_RGB, TRUE, 8, app.icon.pixmap_variant.w,
        app.icon.pixmap_variant.h, 4 * app.icon.pixmap_variant.w);
    GdkPaintable *paintable = GDK_PAINTABLE(gdk_texture_new_for_pixbuf(pixbuf));
    icon_with_menu->icon = gtk_image_new_from_paintable(paintable);
    break;
  }
  case IO_TrayIcon_None: {
    icon_with_menu->icon = gtk_image_new_from_gicon(get_question_mark_icon());
    break;
  }
  }

  GMenu *menu = g_menu_new();

  for (size_t i = 0; i < app.items.len; i++) {
    GMenuItem *item = g_menu_item_new(app.items.ptr[i].label, NULL);
    const char *action = "tray.clicked";
    if (app.items.ptr[i].disabled) {
      action = "tray.noop";
    }
    g_menu_item_set_action_and_target_value(
        item, action, g_variant_new_string(app.items.ptr[i].uuid));
    g_menu_append_item(menu, item);
    g_object_unref(item);
  }

  icon_with_menu->menu = gtk_popover_menu_new_from_model(G_MENU_MODEL(menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(icon_with_menu->menu), FALSE);
  gtk_widget_set_parent(icon_with_menu->menu, icon_with_menu->icon);

  GtkGesture *gesture = gtk_gesture_click_new();
  gtk_gesture_single_set_button(GTK_GESTURE_SINGLE(gesture),
                                3 /* right click */);
  gtk_widget_add_controller(icon_with_menu->icon,
                            GTK_EVENT_CONTROLLER(gesture));

  g_signal_connect(gesture, "pressed", G_CALLBACK(_(open_popover)),
                   icon_with_menu);
}

static void _(cleanup_prev_icons)() {
  while (gtk_widget_get_first_child(_(widget)) != NULL) {
    GtkWidget *image = gtk_widget_get_first_child(_(widget));
    GtkWidget *popover = gtk_widget_get_first_child(image);
    gtk_widget_unparent(popover);
    gtk_widget_unparent(image);
  }

  for (size_t i = 0; i < ICONS_COUNT; i++) {
    _(icons)[i].icon = NULL;
    _(icons)[i].menu = NULL;
  }
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_Tray: {
    _(cleanup_prev_icons)();

    IO_CArray_TrayApp apps = event->tray.list;
    for (size_t i = 0; i < ICONS_COUNT; i++) {
      if (i < apps.len) {
        IO_TrayApp app = apps.ptr[i];
        icon_with_menu_t *icon_with_menu = &(_(icons)[i]);
        _(set_tray_item)(icon_with_menu, app);
        gtk_box_append(GTK_BOX(_(widget)), icon_with_menu->icon);
      }
    }
    break;
  }

  default:
    break;
  }
}

static void _(activate)(void) { layer_shell_io_subscribe(_(on_io_event)); }

widget_t TRAY_WIDGET = {.init = _(init), .activate = _(activate)};
