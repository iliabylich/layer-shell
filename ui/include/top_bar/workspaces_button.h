#pragma once

#include <gtk/gtk.h>

GtkWidget *workspaces_button_new(GtkWidget *workspaces, size_t idx);
void workspaces_button_make_active(GtkWidget *button);
void workspaces_button_make_inactive(GtkWidget *button);
