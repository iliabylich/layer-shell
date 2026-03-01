#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(IOModel, io_model, IO, MODEL, GObject)

#define IO_MODEL(obj)                                                          \
  G_TYPE_CHECK_INSTANCE_CAST(obj, io_model_get_type(), IOModel)

IOModel *io_model_new(void);

void io_model_set_clock_unix_seconds(IOModel *self, int64_t unix_seconds);
void io_model_set_download_speed(IOModel *self, const char *text);
void io_model_set_upload_speed(IOModel *self, const char *text);

void io_model_set_workspaces(IOModel *self,
                             struct IO_FFIArray_HyprlandWorkspace data);
void io_model_set_weather(IOModel *self, float temperature,
                          IO_WeatherCode code);
void io_model_set_language(IOModel *self, const char *lang);
void io_model_set_cpu(IOModel *self, IO_FFIArray_u8 data);
void io_model_set_memory(IOModel *self, float used, float total);
void io_model_set_network_ssid(IOModel *self, const char *ssid);
void io_model_set_network_strength(IOModel *self, uint8_t strength);

void io_model_tray_add_app(IOModel *self, const char *service, IO_TrayIcon icon,
                           IO_FFIArray_TrayItem items);
void io_model_tray_remove_app(IOModel *self, const char *service);
void io_model_tray_set_icon(IOModel *self, const char *service,
                            IO_TrayIcon icon);
void io_model_tray_set_menu(IOModel *self, const char *service,
                            IO_FFIArray_TrayItem items);
