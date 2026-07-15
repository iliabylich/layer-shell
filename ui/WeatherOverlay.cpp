#include "WeatherOverlay.hpp"
#include "Overlay.hpp"
#include "UiModel.hpp"

#include <LayerShellQt/Window>
#include <QDateTime>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QKeyEvent>
#include <QLabel>
#include <QVBoxLayout>
#include <QWindow>

enum class WeatherColumnType { TextLeft, TextRight, Icon };

class WeatherGridWithHeader : public QVBoxLayout {
public:
  WeatherGridWithHeader(uint32_t rows,
                        const QVector<WeatherColumnType> &columns,
                        const QString &header_text)
      : QVBoxLayout() {
    setContentsMargins(0, 0, 0, 0);
    setSpacing(6);
    auto *header = new QLabel(header_text);
    header->setAlignment(Qt::AlignCenter);
    addWidget(header);

    grid = new QGridLayout();
    grid->setContentsMargins(0, 0, 0, 0);
    grid->setHorizontalSpacing(0);
    grid->setVerticalSpacing(0);
    grid->setColumnMinimumWidth(0, 60);
    grid->setColumnMinimumWidth(1, 60);
    grid->setColumnMinimumWidth(2, 36);

    for (uint32_t row = 0; row < rows; row++) {
      for (uint32_t col = 0; col < columns.length(); col++) {
        auto *label = new QLabel("");
        switch (columns[col]) {
        case WeatherColumnType::TextLeft: {
          label->setAlignment(Qt::AlignLeft | Qt::AlignVCenter);
          break;
        }
        case WeatherColumnType::TextRight: {
          label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
          break;
        }
        case WeatherColumnType::Icon: {
          label->setObjectName("icon");
          label->setAlignment(Qt::AlignCenter);
        }
        }
        grid->addWidget(label, row, col);
      }
    }

    addLayout(grid);
    addStretch(1);
  }

  void setTextAt(uint32_t row, uint32_t column, const QString &text) {
    auto *label =
        qobject_cast<QLabel *>(grid->itemAtPosition(row, column)->widget());
    label->setText(text);
  }

  void setIconAt(uint32_t row, uint32_t column, const QString &text,
                 const QString &tooltip) {
    auto *label =
        qobject_cast<QLabel *>(grid->itemAtPosition(row, column)->widget());
    label->setText(text);
    label->setToolTip(tooltip);
  }

private:
  QGridLayout *grid = nullptr;
};

//

QString format_hour(qint64 unix_seconds) {
  return QDateTime::fromSecsSinceEpoch(unix_seconds).toString("HH:mm");
}

QString format_day(qint64 unix_seconds) {
  return QDateTime::fromSecsSinceEpoch(unix_seconds).toString("MMM-dd");
}

QString format_temperature(double temperature) {
  return QStringLiteral("%1℃").arg(temperature, 5, 'f', 1);
}

class HourlyWeatherGrid : public WeatherGridWithHeader {
public:
  static constexpr size_t ROWS = Event::Weather::OnHour::COUNT;
  static constexpr std::array<WeatherColumnType, 3> COLUMNS = {
      WeatherColumnType::TextLeft,
      WeatherColumnType::TextRight,
      WeatherColumnType::Icon,
  };
  static constexpr const char *HEADER = "Hourly";

  HourlyWeatherGrid()
      : WeatherGridWithHeader(
            ROWS, QVector<WeatherColumnType>(COLUMNS.begin(), COLUMNS.end()),
            HEADER) {}

  void update(const std::array<WeatherHourForecast,
                               Event::Weather::OnHour::COUNT> &data) {
    for (size_t row = 0; row < ROWS; row++) {
      const WeatherHourForecast &item = data[row];
      setTextAt(row, 0, format_hour(item.unix_seconds));
      setTextAt(row, 1, format_temperature(item.temperature));
      setIconAt(row, 2, item.icon, item.description);
    }
  }
};

class DailyWeatherGrid : public WeatherGridWithHeader {
public:
  static constexpr size_t ROWS = Event::Weather::OnDay::COUNT;
  static constexpr std::array<WeatherColumnType, 4> COLUMNS = {
      WeatherColumnType::TextLeft,
      WeatherColumnType::TextRight,
      WeatherColumnType::TextRight,
      WeatherColumnType::Icon,
  };
  static constexpr const char *HEADER = "Daily";

  DailyWeatherGrid()
      : WeatherGridWithHeader(
            ROWS, QVector<WeatherColumnType>(COLUMNS.begin(), COLUMNS.end()),
            HEADER) {}

  void update(const std::array<WeatherDayForecast, ROWS> &data) {
    for (size_t row = 0; row < ROWS; row++) {
      const WeatherDayForecast &item = data[row];
      setTextAt(row, 0, format_day(item.unix_seconds));
      setTextAt(row, 1, format_temperature(item.temperature_min));
      setTextAt(row, 2, format_temperature(item.temperature_max));
      setIconAt(row, 3, item.icon, item.description);
    }
  }
};

//

constexpr int WeatherOverlayWidth = 420;
constexpr int WeatherOverlayHeight = 330;

WeatherOverlay::WeatherOverlay(UiModel *model) : Overlay(model) {
  auto *layout = initCenteredLayout(
      QSize(WeatherOverlayWidth, WeatherOverlayHeight), "WeatherOverlay");
  layout->setContentsMargins(10, 10, 10, 10);
  layout->setSpacing(50);

  hourly = new HourlyWeatherGrid();
  layout->addLayout(hourly, 0);

  daily = new DailyWeatherGrid();
  layout->addLayout(daily, 0);

  connect(model, &UiModel::weatherChanged, this,
          [this]([[maybe_unused]] const QString &summary,
                 const std::array<WeatherHourForecast,
                                  Event::Weather::OnHour::COUNT> &hourly_data,
                 const std::array<WeatherDayForecast,
                                  Event::Weather::OnDay::COUNT> &daily_data) {
            hourly->update(hourly_data);
            daily->update(daily_data);
          });

  layer->setScope("LayerShell/Weather");
  layer->setLayer(LayerShellQt::Window::LayerOverlay);
  layer->setKeyboardInteractivity(
      LayerShellQt::Window::KeyboardInteractivityExclusive);
  setCloseOnEscape(true);
}
