#pragma once

#include "Overlay.hpp"

class QLabel;
class QTimer;
class UiModel;
class VolumeBar;

class SoundOverlay : public Overlay {
  Q_OBJECT

public:
  explicit SoundOverlay(UiModel *model);

private:
  QLabel *icon_ = nullptr;
  VolumeBar *bar_ = nullptr;
  QTimer *hide_timer_ = nullptr;
};
