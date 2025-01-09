#ifndef UTILS_H
#define UTILS_H

#include <gtk/gtk.h>
#include <stdint.h>

typedef void (*init_window_f)(void);
typedef void (*toggle_window_f)(void);
typedef void (*activate_window_f)(GApplication *app);
typedef void (*move_window_f)(uint32_t margin_left, uint32_t margin_top);
typedef uint32_t (*get_width_f)(void);

typedef struct {
  init_window_f init;
  activate_window_f activate;
  toggle_window_f toggle;
  move_window_f move;
  get_width_f width;
} window_t;

void flip_window_visibility(GtkWindow *window);
void move_layer_window(GtkWindow *window, uint32_t margin_left,
                       uint32_t margin_top);

#endif // UTILS_H
