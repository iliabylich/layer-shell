#include "ui/tray_popover.h"
#include "ui/assertions.h"
#include "ui/logger.h"

LOGGER("TrayPopover", 3)

enum {
  SIGNAL_TRIGGERED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _TrayPopover {
  GtkWidget parent_instance;

  GtkWidget *root;
  GMenu *menu;
  GSimpleActionGroup *action_group;
};

G_DEFINE_TYPE(TrayPopover, tray_popover, GTK_TYPE_WIDGET)

#define NAMESPACE "tray"
#define UUID_KEY "uuid"

static void on_activate(GSimpleAction *action, GVariant *, TrayPopover *self) {
  const char *uuid = g_object_get_data(G_OBJECT(action), UUID_KEY);
  g_signal_emit(self, signals[SIGNAL_TRIGGERED], 0, uuid);
}

static void tray_popover_init(TrayPopover *self) {
  LOG("init");

  self->menu = g_menu_new();
  self->action_group = g_simple_action_group_new();

  self->root = gtk_popover_menu_new_from_model(G_MENU_MODEL(self->menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(self->root), false);
  gtk_popover_menu_set_flags(GTK_POPOVER_MENU(self->root),
                             GTK_POPOVER_MENU_NESTED);

  gtk_widget_insert_action_group(GTK_WIDGET(self), NAMESPACE,
                                 G_ACTION_GROUP(self->action_group));
  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void tray_popover_dispose(GObject *object) {
  LOG("dispose");

  TrayPopover *self = TRAY_POPOVER(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(tray_popover_parent_class)->dispose(object);
}

static void tray_popover_class_init(TrayPopoverClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = tray_popover_dispose;
  signals[SIGNAL_TRIGGERED] = g_signal_new_class_handler(
      "triggered", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *tray_popover_new() {
  return g_object_new(tray_popover_get_type(), NULL);
}

void tray_popover_open(TrayPopover *self) {
  gtk_popover_popup(GTK_POPOVER(self->root));
}

static void visit_all(TrayPopover *self, IO_FFIArray_TrayItem items,
                      GMenu *menu);
static void visit(TrayPopover *self, IO_TrayItem tray_item, GMenu *menu);

void tray_popover_update(TrayPopover *self, IO_FFIArray_TrayItem items) {
  g_menu_remove_all(self->menu);

  visit_all(self, items, self->menu);
}

static void visit_all(TrayPopover *self, IO_FFIArray_TrayItem items,
                      GMenu *menu) {
  for (size_t idx = 0; idx < items.len; idx++) {
    IO_TrayItem child = items.ptr[idx];
    visit(self, child, menu);
  }
}

static void add_action(TrayPopover *self, uint32_t id, const char *uuid,
                       const GVariantType *parameter_type, GVariant *state) {
  char name[100];
  checked_fmt(name, "%d", id);
  GSimpleAction *action =
      g_simple_action_new_stateful(name, parameter_type, state);

  g_object_set_data_full(G_OBJECT(action), UUID_KEY, strdup(uuid), free);

  g_signal_connect(action, "activate", G_CALLBACK(on_activate), self);
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

static void visit_regular(TrayPopover *self, Regular regular, GMenu *menu) {
  add_menu_item(menu, regular.id, regular.label, NULL);
  add_action(self, regular.id, regular.uuid, NULL, NULL);
}

static void visit_disabled(TrayPopover *, Disabled disabled, GMenu *menu) {
  add_menu_item(menu, disabled.id, disabled.label, NULL);
}

static void visit_checkbox(TrayPopover *self, Checkbox checkbox, GMenu *menu) {
  add_menu_item(menu, checkbox.id, checkbox.label, NULL);
  add_action(self, checkbox.id, checkbox.uuid, NULL,
             g_variant_new_boolean(checkbox.checked));
}

static void visit_radio(TrayPopover *self, Radio radio, GMenu *menu) {
  add_menu_item(menu, radio.id, radio.label, g_variant_new_boolean(true));
  add_action(self, radio.id, radio.uuid, G_VARIANT_TYPE_BOOLEAN,
             g_variant_new_boolean(radio.selected));
}

static void visit_nested(TrayPopover *self, Nested nested, GMenu *menu) {
  GMenu *submenu = g_menu_new();
  visit_all(self, nested.children, submenu);

  GMenuItem *menu_item =
      g_menu_item_new_submenu(nested.label, G_MENU_MODEL(submenu));
  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

static void visit_section(TrayPopover *self, Section nested, GMenu *menu) {
  GMenu *section = g_menu_new();
  visit_all(self, nested.children, section);
  g_menu_append_section(menu, NULL, G_MENU_MODEL(section));
}

static void visit(TrayPopover *self, IO_TrayItem tray_item, GMenu *menu) {
  switch (tray_item.tag) {
  case IO_TrayItem_Regular: {
    visit_regular(self, tray_item.regular, menu);
    return;
  }
  case IO_TrayItem_Disabled: {
    visit_disabled(self, tray_item.disabled, menu);
    return;
  }
  case IO_TrayItem_Checkbox: {
    visit_checkbox(self, tray_item.checkbox, menu);
    return;
  }
  case IO_TrayItem_Radio: {
    visit_radio(self, tray_item.radio, menu);
    return;
  }
  case IO_TrayItem_Nested: {
    visit_nested(self, tray_item.nested, menu);
    return;
  }
  case IO_TrayItem_Section: {
    visit_section(self, tray_item.section, menu);
    return;
  }
  }
}
