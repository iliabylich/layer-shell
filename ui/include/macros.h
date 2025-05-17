#define NULL_TERMINATED_STRING_LIST(...)                                       \
  (const char *[]) { __VA_ARGS__, NULL }

#define CSS NULL_TERMINATED_STRING_LIST

#define VTE_CMD NULL_TERMINATED_STRING_LIST

#define TRAY_ITEM_IS_NESTED(tray_item)                                         \
  strcmp(tray_item.children_display, "submenu") == 0
#define TRAY_ITEM_IS_DISABLED(tray_item) !tray_item.enabled
#define TRAY_ITEM_IS_CHECKBOX(tray_item)                                       \
  strcmp(tray_item.toggle_type, "checkmark") == 0
#define TRAY_ITEM_IS_RADIO(tray_item)                                          \
  strcmp(tray_item.toggle_type, "radio") == 0

#define BLP_BUILDER(name)                                                      \
  static GtkBuilder *_builder = NULL;                                          \
  static GtkBuilder *builder() {                                               \
    if (_builder == NULL) {                                                    \
      _builder = gtk_builder_new_from_string((const char *)name##_xml,         \
                                             name##_xml_len);                  \
    }                                                                          \
    return _builder;                                                           \
  }                                                                            \
  static GtkWidget *builder_get_object(const char *id) {                       \
    return GTK_WIDGET(gtk_builder_get_object(builder(), id));                  \
  }
