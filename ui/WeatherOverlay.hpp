#pragma once

#include "Overlay.hpp"

class UiModel;
class HourlyWeatherGrid;
class DailyWeatherGrid;

class WeatherOverlay : public Overlay {
  Q_OBJECT

public:
  explicit WeatherOverlay(UiModel *model);

private:
  HourlyWeatherGrid *hourly = nullptr;
  DailyWeatherGrid *daily = nullptr;
};
