#pragma once

#include <gtk/gtk.h>

typedef void (*on_lock_clicked_f)();
typedef void (*on_reboot_clicked_f)();
typedef void (*on_shutdown_clicked_f)();
typedef void (*on_logout_clicked_f)();

GtkWidget *session_init(GtkApplication *app,
                        on_lock_clicked_f lock_clicked_callback,
                        on_reboot_clicked_f reboot_clicked_callback,
                        on_shutdown_clicked_f shutdown_clicked_callback,
                        on_logout_clicked_f logout_clicked_callback);
void session_toggle(GtkWidget *session);
