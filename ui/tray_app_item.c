#include "ui/tray_app_item.h"
#include "ui/assertions.h"
#include "ui/logger.h"

LOGGER("TrayAppItem", 2)

enum {
  SIGNAL_TRIGGERED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

enum {
  PROP_PAINTABLE = 1,
  PROP_POPOVER,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

struct _TrayAppItem {
  GObject parent_instance;

  char *service;
  GdkPaintable *paintable;
  GtkWidget *popover;
  GMenu *menu;
  GSimpleActionGroup *action_group;
};

G_DEFINE_TYPE(TrayAppItem, tray_app_item, G_TYPE_OBJECT)

#define NAMESPACE "tray"
#define UUID_KEY "uuid"
#define EMPTY_ICON_NAME "process-stop"

static void set_paintable(TrayAppItem *self, GdkPaintable *paintable) {
  g_set_object(&self->paintable, paintable);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_PAINTABLE]);
}

static void icon_from_path(TrayAppItem *self, const char *path) {
  GdkTexture *texture = gdk_texture_new_from_filename(path, NULL);
  if (texture) {
    set_paintable(self, GDK_PAINTABLE(texture));
    g_object_unref(texture);
  }
}

static void icon_from_name(TrayAppItem *self, const char *name) {
  GtkIconTheme *theme =
      gtk_icon_theme_get_for_display(gdk_display_get_default());
  GtkIconPaintable *icon = gtk_icon_theme_lookup_icon(
      theme, name, NULL, 24, 1, GTK_TEXT_DIR_NONE, GTK_ICON_LOOKUP_PRELOAD);
  set_paintable(self, GDK_PAINTABLE(icon));
  g_object_unref(icon);
}

static void icon_from_pixmap(TrayAppItem *self, uint8_t *data, size_t size,
                             uint32_t w, uint32_t h) {
  GBytes *bytes = g_bytes_new(data, size);
  GdkTexture *texture =
      gdk_memory_texture_new(w, h, GDK_MEMORY_R8G8B8A8, bytes, 4 * w);
  set_paintable(self, GDK_PAINTABLE(texture));
  g_bytes_unref(bytes);
  g_object_unref(texture);
}

static void icon_from_unset(TrayAppItem *self) {
  icon_from_name(self, EMPTY_ICON_NAME);
}

static void activate(GSimpleAction *action, GVariant *, TrayAppItem *self) {
  const char *uuid = g_object_get_data(G_OBJECT(action), UUID_KEY);
  g_signal_emit(self, signals[SIGNAL_TRIGGERED], 0, uuid);
}

static void add_action(TrayAppItem *self, uint32_t id, const char *uuid,
                       const GVariantType *parameter_type, GVariant *state) {
  char name[100];
  checked_fmt(name, "%d", id);
  GSimpleAction *action =
      g_simple_action_new_stateful(name, parameter_type, state);
  g_object_set_data_full(G_OBJECT(action), UUID_KEY, strdup(uuid), free);
  g_signal_connect(action, "activate", G_CALLBACK(activate), self);
  g_action_map_add_action(G_ACTION_MAP(self->action_group), G_ACTION(action));
}

static void add_menu_item(GMenu *menu, uint32_t id, const char *label,
                          GVariant *target_value) {
  GMenuItem *menu_item = g_menu_item_new(label, NULL);
  char action[100];
  checked_fmt(action, "%s.%d", NAMESPACE, id);
  g_menu_item_set_action_and_target_value(menu_item, action, target_value);
  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

typedef IO_TrayItem_IO_Regular_Body Regular;
typedef IO_TrayItem_IO_Disabled_Body Disabled;
typedef IO_TrayItem_IO_Checkbox_Body Checkbox;
typedef IO_TrayItem_IO_Radio_Body Radio;
typedef IO_TrayItem_IO_Nested_Body Nested;
typedef IO_TrayItem_IO_Section_Body Section;

static void visit_all(TrayAppItem *self, IO_FFIArray_TrayItem items,
                      GMenu *menu);
static void visit(TrayAppItem *self, IO_TrayItem tray_item, GMenu *menu);

static void visit_regular(TrayAppItem *self, Regular regular, GMenu *menu) {
  add_menu_item(menu, regular.id, regular.label, NULL);
  add_action(self, regular.id, regular.uuid, NULL, NULL);
}

static void visit_disabled(TrayAppItem *, Disabled disabled, GMenu *menu) {
  add_menu_item(menu, disabled.id, disabled.label, NULL);
}

static void visit_checkbox(TrayAppItem *self, Checkbox checkbox, GMenu *menu) {
  add_menu_item(menu, checkbox.id, checkbox.label, NULL);
  add_action(self, checkbox.id, checkbox.uuid, NULL,
             g_variant_new_boolean(checkbox.checked));
}

static void visit_radio(TrayAppItem *self, Radio radio, GMenu *menu) {
  add_menu_item(menu, radio.id, radio.label, g_variant_new_boolean(true));
  add_action(self, radio.id, radio.uuid, G_VARIANT_TYPE_BOOLEAN,
             g_variant_new_boolean(radio.selected));
}

static void visit_nested(TrayAppItem *self, Nested nested, GMenu *menu) {
  GMenu *submenu = g_menu_new();
  visit_all(self, nested.children, submenu);
  GMenuItem *menu_item =
      g_menu_item_new_submenu(nested.label, G_MENU_MODEL(submenu));
  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

static void visit_section(TrayAppItem *self, Section section, GMenu *menu) {
  GMenu *sub = g_menu_new();
  visit_all(self, section.children, sub);
  g_menu_append_section(menu, NULL, G_MENU_MODEL(sub));
}

static void visit_all(TrayAppItem *self, IO_FFIArray_TrayItem items,
                      GMenu *menu) {
  for (size_t idx = 0; idx < items.len; idx++) {
    visit(self, items.ptr[idx], menu);
  }
}

static void visit(TrayAppItem *self, IO_TrayItem tray_item, GMenu *menu) {
  switch (tray_item.tag) {
  case IO_TrayItem_Regular:
    visit_regular(self, tray_item.regular, menu);
    return;
  case IO_TrayItem_Disabled:
    visit_disabled(self, tray_item.disabled, menu);
    return;
  case IO_TrayItem_Checkbox:
    visit_checkbox(self, tray_item.checkbox, menu);
    return;
  case IO_TrayItem_Radio:
    visit_radio(self, tray_item.radio, menu);
    return;
  case IO_TrayItem_Nested:
    visit_nested(self, tray_item.nested, menu);
    return;
  case IO_TrayItem_Section:
    visit_section(self, tray_item.section, menu);
    return;
  }
}

static void tray_app_item_get_property(GObject *object, guint property_id,
                                       GValue *value, GParamSpec *pspec) {
  TrayAppItem *self = TRAY_APP_ITEM(object);
  switch (property_id) {
  case PROP_PAINTABLE:
    g_value_set_object(value, self->paintable);
    break;
  case PROP_POPOVER:
    g_value_set_object(value, self->popover);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void tray_app_item_finalize(GObject *object) {
  TrayAppItem *self = TRAY_APP_ITEM(object);
  g_free(self->service);
  g_clear_object(&self->paintable);
  g_clear_object(&self->menu);
  g_clear_object(&self->action_group);
  G_OBJECT_CLASS(tray_app_item_parent_class)->finalize(object);
}

static void tray_app_item_init(TrayAppItem *self) {
  LOG("init");

  self->menu = g_menu_new();
  self->action_group = g_simple_action_group_new();

  self->popover = gtk_popover_menu_new_from_model(G_MENU_MODEL(self->menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(self->popover), false);
  gtk_popover_menu_set_flags(GTK_POPOVER_MENU(self->popover),
                             GTK_POPOVER_MENU_NESTED);
  gtk_widget_insert_action_group(self->popover, NAMESPACE,
                                 G_ACTION_GROUP(self->action_group));
}

static void tray_app_item_class_init(TrayAppItemClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = tray_app_item_get_property;
  object_class->finalize = tray_app_item_finalize;

  properties[PROP_PAINTABLE] = g_param_spec_object(
      "paintable", NULL, NULL, GDK_TYPE_PAINTABLE, G_PARAM_READABLE);
  properties[PROP_POPOVER] = g_param_spec_object(
      "popover", NULL, NULL, GTK_TYPE_WIDGET, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  signals[SIGNAL_TRIGGERED] = g_signal_new_class_handler(
      "triggered", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
}

TrayAppItem *tray_app_item_new(const char *service, IO_TrayIcon icon,
                               IO_FFIArray_TrayItem items) {
  TrayAppItem *self = g_object_new(tray_app_item_get_type(), NULL);
  self->service = g_strdup(service);
  tray_app_item_update_icon(self, icon);
  tray_app_item_update_menu(self, items);
  return self;
}

const char *tray_app_item_get_service(TrayAppItem *self) {
  return self->service;
}

void tray_app_item_update_icon(TrayAppItem *self, IO_TrayIcon icon) {
  switch (icon.tag) {
  case IO_TrayIcon_Path:
    icon_from_path(self, icon.path.path);
    break;
  case IO_TrayIcon_Name:
    icon_from_name(self, icon.name.name);
    break;
  case IO_TrayIcon_Pixmap:
    icon_from_pixmap(self, icon.pixmap.bytes.ptr, icon.pixmap.bytes.len,
                     icon.pixmap.width, icon.pixmap.height);
    break;
  case IO_TrayIcon_Unset:
    icon_from_unset(self);
    break;
  }
}

void tray_app_item_update_menu(TrayAppItem *self, IO_FFIArray_TrayItem items) {
  g_menu_remove_all(self->menu);
  g_clear_object(&self->action_group);
  self->action_group = g_simple_action_group_new();
  gtk_widget_insert_action_group(self->popover, NAMESPACE,
                                 G_ACTION_GROUP(self->action_group));
  visit_all(self, items, self->menu);
}
