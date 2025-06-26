#pragma once

#include <gtk/gtk.h>

typedef void (*on_session_btn_clicked_f)();

GtkWidget *session_init(GtkApplication *app,
                        on_session_btn_clicked_f lock_clicked_callback,
                        on_session_btn_clicked_f reboot_clicked_callback,
                        on_session_btn_clicked_f shutdown_clicked_callback,
                        on_session_btn_clicked_f logout_clicked_callback);
void session_toggle(GtkWidget *session);
