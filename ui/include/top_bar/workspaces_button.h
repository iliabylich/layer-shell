#pragma once

#include "ui/include/top_bar/workspaces.h"

GtkWidget *workspaces_button_new(size_t num, workspace_change_f on_click);
size_t workspaces_button_get_number(GtkWidget *button);
void workspaces_button_make_active(GtkWidget *button);
void workspaces_button_make_inactive(GtkWidget *button);
