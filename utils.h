#ifndef UTILS_H
#define UTILS_H

#include <gtk/gtk.h>

typedef void (*init_window_f)(void);
typedef void (*toggle_window_f)(void);
typedef void (*activate_window_f)(GApplication *app);
typedef void (*move_window_f)(int x, int y);

typedef struct {
  init_window_f init;
  activate_window_f activate;
  toggle_window_f toggle;
  move_window_f move;
  int width;
} window_t;

void flip_window_visibility(GtkWindow *window);
void move_layer_window(GtkWindow *window, int x, int y);
void window_set_width_request(GtkWindow *window, int width);
void window_set_height_request(GtkWindow *window, int height);

typedef void (*init_widget_f)(void);
typedef void (*active_widget_f)(void);
typedef GtkWidget *(*main_widget_f)(void);

typedef struct {
  init_widget_f init;
  active_widget_f activate;
  main_widget_f main_widget;
} widget_t;

#endif // UTILS_H
