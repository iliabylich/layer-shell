#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Session, session, SESSION, WINDOW, GtkWindow)

Session *session_new(GtkApplication *app);
