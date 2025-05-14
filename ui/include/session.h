#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Session, session, SESSION, WINDOW, GtkWindow)

GtkWidget *session_new(GtkApplication *app);

#define SESSION_TYPE session_get_type()
#define SESSION(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, SESSION_TYPE, Session)
