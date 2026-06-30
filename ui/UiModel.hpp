#pragma once

#include "Event.hpp"
#include <QObject>
#include <QString>
#include <QVector>

class QIcon;
class QSocketNotifier;
struct IO_Event;
struct IO_Event_IO_Weather_Body;

struct WeatherHourForecast {
  qint64 unix_seconds;
  double temperature;
  QString icon;
  QString description;
};

struct WeatherDayForecast {
  qint64 unix_seconds;
  double temperature_min;
  double temperature_max;
  QString icon;
  QString description;
};

class UiModel : public QObject {
  Q_OBJECT

public:
  explicit UiModel(QObject *parent = nullptr);
  ~UiModel() override;

  IO_IO *getIO() const;

  void changeWallpaper();
  void lock();
  void logout();
  void reboot();
  void shutdown();
  void spawnBluetoothEditor();
  void spawnWifiEditor();
  void spawnSystemMonitor();
  void triggerTrayItem(const QString &uuid);

Q_SIGNALS:
  void timeTextChanged(const QString &text);
  void cpuTextChanged(const QString &text);
  void kbModChanged(const QString &icon, const QString &text);
  void languageTextChanged(const QString &text);
  void memoryTextChanged(const QString &text);
  void networkDownloadSpeedChanged(const QString &text);
  void networkUploadSpeedChanged(const QString &text);
  void networkSsidAndStrengthChanged(const QString &text);
  void sessionToggleRequested();
  void soundChanged(uint32_t volume, const QString &icon);
  void trayAppAdded(const QString &app_id, const QIcon &icon,
                    const QVector<Event::Tray::MenuItem> &items);
  void trayAppIconUpdated(const QString &app_id, const QIcon &icon);
  void trayAppMenuUpdated(const QString &app_id,
                          const QVector<Event::Tray::MenuItem> &items);
  void trayAppRemoved(const QString &app_id);
  void weatherChanged(
      const QString &summary,
      const std::array<WeatherDayForecast, Event::Weather::OnDay::COUNT> &daily,
      const std::array<WeatherHourForecast, Event::Weather::OnHour::COUNT>
          &hourly);
  void exitRequested();

private:
  static void eventReceived(const IO_Event *event, void *data);
  void handleEvent(const struct IO_Event &event);

  void operator()(const Event::Memory &e);
  void operator()(const Event::CPU &e);
  void operator()(const Event::Time &e);
  void operator()(const Event::Language &e);
  void operator()(const Event::Weather &e);
  void operator()(const Event::Network &e);
  void operator()(const Event::UploadSpeed &e);
  void operator()(const Event::DownloadSpeed &e);
  void operator()(const Event::Tray::AppAdded &e);
  void operator()(const Event::Tray::AppIconUpdated &e);
  void operator()(const Event::Tray::AppMenuUpdated &e);
  void operator()(const Event::Tray::AppRemoved &e);
  void operator()(const Event::KbModToggled &e);
  void operator()(const Event::Sound &e);
  void operator()(const Event::ToggleSessionScreen &e);
  void operator()(const Event::Exit &e);

  IO_IO *io;
  QSocketNotifier *io_notifier_ = nullptr;
};
