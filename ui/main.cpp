#include "Config.hpp"
#include "KbModOverlay.hpp"
#include "SessionOverlay.hpp"
#include "SoundOverlay.hpp"
#include "TerminalOverlay.hpp"
#include "TopBar.hpp"
#include "UiModel.hpp"
#include "WeatherOverlay.hpp"
#include "style.scss.xxd"

#include <QApplication>

int main(int argc, char **argv) {
  QApplication app(argc, argv);

  auto style = QString::fromUtf8(reinterpret_cast<const char *>(style_scss),
                                 style_scss_len);
  app.setStyleSheet(style);

  auto model = new UiModel();

  KbModOverlay kb_mod_overlay(model);
  SessionOverlay session_overlay(model);
  SoundOverlay sound_overlay(model);
  TerminalOverlay terminal_overlay(model, "LayerShell/Terminal",
                                   Config::getTerminalCommand());
  TerminalOverlay ping_overlay(model, "LayerShell/Ping",
                               Config::getPingCommand());
  WeatherOverlay weather_overlay(model);

  TopBar top_bar(model);

  QObject::connect(&top_bar, &TopBar::weatherClicked, &weather_overlay,
                   &WeatherOverlay::toggle);
  QObject::connect(&top_bar, &TopBar::terminalClicked, &terminal_overlay,
                   &TerminalOverlay::toggle);
  QObject::connect(&top_bar, &TopBar::pingClicked, &ping_overlay,
                   &TerminalOverlay::toggle);
  QObject::connect(&top_bar, &TopBar::powerClicked, &session_overlay,
                   &SessionOverlay::toggle);
  QObject::connect(model, &UiModel::exitRequested, &app, &QApplication::quit);

  return QApplication::exec();
}
