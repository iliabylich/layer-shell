#include "TopBar.hpp"
#include "Config.hpp"
#include "Tray.hpp"
#include "UiModel.hpp"
#include <LayerShellQt/Window>
#include <QHBoxLayout>
#include <QLabel>
#include <QMenu>
#include <QPushButton>

constexpr int WidgetsHeight = 36;
constexpr int Padding = 10;
constexpr int TopBarHeight = WidgetsHeight + Padding;

class __Button : public QPushButton {
public:
  explicit __Button(const QString &text) : QPushButton(text) {
    setCursor(Qt::PointingHandCursor);
    setSizePolicy(QSizePolicy::Minimum, QSizePolicy::Expanding);
  }
};

class __IconButton : public QPushButton {
public:
  explicit __IconButton(const QString &text) : QPushButton(text) {
    setCursor(Qt::PointingHandCursor);
    setSizePolicy(QSizePolicy::Minimum, QSizePolicy::Expanding);
  }
};

class __Label : public QLabel {
public:
  explicit __Label(const QString &text) : QLabel(text) {
    setSizePolicy(QSizePolicy::Minimum, QSizePolicy::Expanding);
  }
};

class ChangeWallpaper : public __IconButton {
public:
  explicit ChangeWallpaper(UiModel *model) : __IconButton("") {
    setObjectName("ChangeWallpaper");

    QObject::connect(this, &ChangeWallpaper::clicked, model,
                     &UiModel::changeWallpaper);
  }
};

class Tray : public QFrame {
public:
  explicit Tray(UiModel *model) : QFrame() {
    setObjectName("Tray");
    setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Expanding);

    auto layout = new QHBoxLayout(this);
    layout->setContentsMargins(10, 0, 10, 0);
    layout->setSpacing(0);

    auto registry = new TrayRegistry(model, this, layout);

    QObject::connect(model, &UiModel::trayAppAdded, registry,
                     &TrayRegistry::add);
    QObject::connect(model, &UiModel::trayAppIconUpdated, registry,
                     &TrayRegistry::updateIcon);
    QObject::connect(model, &UiModel::trayAppMenuUpdated, registry,
                     &TrayRegistry::updateMenu);
    QObject::connect(model, &UiModel::trayAppRemoved, registry,
                     &TrayRegistry::remove);
  }
};

class Weather : public __Button {
public:
  Weather(UiModel *model, TopBar *parent) : __Button("--") {
    setObjectName("Weather");

    QObject::connect(model, &UiModel::weatherChanged, this, &Weather::setText);
    QObject::connect(this, &Weather::clicked, parent, &TopBar::weatherClicked);
  }
};

class Terminal : public __Button {
public:
  explicit Terminal(TopBar *parent, IO *io)
      : __Button(Config::getTerminalLabel(io)) {
    setObjectName("Terminal");

    QObject::connect(this, &Terminal::clicked, parent,
                     &TopBar::terminalClicked);
  }
};

class Language : public __Label {
public:
  explicit Language(UiModel *model) : __Label("--") {
    setAlignment(Qt::AlignCenter);
    setObjectName("Language");
    QObject::connect(model, &UiModel::languageTextChanged, this,
                     &Language::setText);
  }
};

class CPU : public __Label {
public:
  explicit CPU(UiModel *model) : __Label("--") {
    setAlignment(Qt::AlignCenter);
    setObjectName("Cpu");
    setTextFormat(Qt::RichText);
    QObject::connect(model, &UiModel::cpuTextChanged, this, &CPU::setText);
  }
};

class Memory : public __Button {
public:
  explicit Memory(UiModel *model) : __Button("--") {
    setObjectName("Memory");
    QObject::connect(model, &UiModel::memoryTextChanged, this,
                     &Memory::setText);
    QObject::connect(this, &Memory::clicked, model,
                     &UiModel::spawnSystemMonitor);
  }
};

class Network : public __Button {
public:
  Network(UiModel *model, TopBar *parent) : __Button("") {
    setObjectName("Network");
    auto *layout = new QHBoxLayout(this);
    layout->setContentsMargins(10, 0, 10, 0);
    layout->setSpacing(0);

    auto *label = new QLabel("--");
    label->setObjectName("NetworkName");
    label->setAlignment(Qt::AlignCenter);
    layout->addWidget(label);
    QObject::connect(model, &UiModel::networkSsidAndStrengthChanged, label,
                     &QLabel::setText);

    auto *icon = new QLabel("");
    icon->setObjectName("NetworkIcon");
    icon->setAlignment(Qt::AlignCenter);
    layout->addWidget(icon);

    layout->addSpacing(10);
    auto *separator = new QFrame;
    separator->setObjectName("NetworkSeparator");
    separator->setFrameShape(QFrame::VLine);
    separator->setFrameShadow(QFrame::Plain);
    separator->setFixedWidth(1);
    layout->addWidget(separator);
    layout->addSpacing(10);

    auto *download = new QLabel("--");
    download->setObjectName("NetworkSpeed");
    download->setAlignment(Qt::AlignCenter);
    layout->addWidget(download);
    QObject::connect(model, &UiModel::networkDownloadSpeedChanged, download,
                     &QLabel::setText);

    auto *download_icon = new QLabel("󰇚");
    download_icon->setObjectName("NetworkIcon");
    download_icon->setAlignment(Qt::AlignCenter);
    layout->addWidget(download_icon);

    auto *upload = new QLabel("--");
    upload->setObjectName("NetworkSpeed");
    upload->setAlignment(Qt::AlignCenter);
    layout->addWidget(upload);
    QObject::connect(model, &UiModel::networkUploadSpeedChanged, upload,
                     &QLabel::setText);

    auto *upload_icon = new QLabel("󰕒");
    upload_icon->setObjectName("NetworkIcon");
    upload_icon->setAlignment(Qt::AlignCenter);
    layout->addWidget(upload_icon);

    auto *menu = new QMenu(this);
    menu->addAction("Settings (iwmenu)", model, &UiModel::spawnWifiEditor);
    QAction *ping_action = menu->addAction("Ping");
    QObject::connect(ping_action, &QAction::triggered, parent,
                     &TopBar::pingClicked);
    setMenu(menu);
  }
};

class Bluetooth : public __IconButton {
public:
  explicit Bluetooth(UiModel *model) : __IconButton("󰂯") {
    setObjectName("Bluetooth");
    QObject::connect(this, &Bluetooth::clicked, model,
                     &UiModel::spawnBluetoothEditor);
  }
};

class Time : public __Label {
public:
  explicit Time(UiModel *model) : __Label("--") {
    setAlignment(Qt::AlignCenter);
    setObjectName("Time");
    QObject::connect(model, &UiModel::timeTextChanged, this, &Time::setText);
  }
};

class Power : public __Button {
public:
  explicit Power(TopBar *parent) : __Button("") {
    setObjectName("Power");
    QObject::connect(this, &Power::clicked, parent, &TopBar::powerClicked);
  }
};

TopBar::TopBar(UiModel *model, IO *io) : Overlay(model), model(model) {
  setObjectName("TopBarOverlay");
  setFixedHeight(TopBarHeight);
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);

  auto *layout = new QHBoxLayout(this);
  layout->setContentsMargins(Padding, Padding, Padding, 0);
  layout->setSpacing(4);

  auto *change_wallpaper = new ChangeWallpaper(model);
  layout->addWidget(change_wallpaper);
  layout->addStretch(1);

  auto *tray = new Tray(model);
  layout->addWidget(tray);

  auto *weather = new Weather(model, this);
  layout->addWidget(weather);

  auto *terminal = new Terminal(this, io);
  layout->addWidget(terminal);

  auto *language = new Language(model);
  layout->addWidget(language);

  auto *cpu = new CPU(model);
  layout->addWidget(cpu);

  auto *memory = new Memory(model);
  layout->addWidget(memory);

  auto *network = new Network(model, this);
  layout->addWidget(network);

  auto *bluetooth = new Bluetooth(model);
  layout->addWidget(bluetooth);

  auto *time = new Time(model);
  layout->addWidget(time);

  auto *power = new Power(this);
  layout->addWidget(power);

  layer->setScope("LayerShell/TopBar");
  layer->setLayer(LayerShellQt::Window::LayerTop);
  layer->setKeyboardInteractivity(
      LayerShellQt::Window::KeyboardInteractivityNone);
  layer->setActivateOnShow(false);
  layer->setAnchors(LayerShellQt::Window::Anchors(
      LayerShellQt::Window::AnchorTop | LayerShellQt::Window::AnchorLeft |
      LayerShellQt::Window::AnchorRight));
  layer->setExclusiveZone(TopBarHeight);

  show();
}
