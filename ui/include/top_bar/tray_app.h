#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"
#include <gtk/gtk.h>

GtkWidget *tray_app_new(IO_TrayApp tray_app, Tray *tray);
