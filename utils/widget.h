#ifndef UTILS_WIDGET_H
#define UTILS_WIDGET_H

#include <gtk/gtk.h>

typedef GtkWidget *(*init_widget_f)(void);
typedef void (*active_widget_f)(void);

typedef struct {
  init_widget_f init;
  active_widget_f activate;
} widget_t;

#endif // UTILS_WIDGET_H
