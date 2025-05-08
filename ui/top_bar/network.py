from gi.repository import Gdk, Gio, GLib, Gtk
from utils.commands import Commands
from utils.context import ctx


class Network(Gtk.Button):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        ctx.pub_sub.subscribe(self)

        self.set_css_classes(["widget", "network", "padded", "clickable"])
        self.set_name("Network")
        self.set_cursor(Gdk.Cursor.new_from_name("pointer"))

        self.label = Gtk.Label.new("-- ")

        self.image = Gtk.Image.new_from_gicon(ctx.icons.wifi)

        self.download_speed_label = Gtk.Label.new("??")
        self.download_speed_label.set_css_classes(["network-speed-label"])
        self.download_speed_icon = Gtk.Image.new_from_gicon(ctx.icons.download)

        self.upload_speed_label = Gtk.Label.new("??")
        self.upload_speed_label.set_css_classes(["network-speed-label"])
        self.upload_speed_icon = Gtk.Image.new_from_gicon(ctx.icons.upload)

        network_wrapper = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        self.set_child(network_wrapper)
        network_wrapper.append(self.label)
        network_wrapper.append(self.image)

        sep = Gtk.Separator.new(Gtk.Orientation.VERTICAL)
        network_wrapper.append(sep)

        network_wrapper.append(self.download_speed_label)
        network_wrapper.append(self.download_speed_icon)
        network_wrapper.append(self.upload_speed_label)
        network_wrapper.append(self.upload_speed_icon)

        self.popover = NetworkPopover()
        self.popover.set_parent(self)

        self.connect("clicked", self.on_click)

    def on_wifi_status(self, event):
        if event.wifi_status is None:
            self.image.hide()
            self.label.set_label("Not connected")
            return
        ssid = event.wifi_status.ssid
        strength = event.wifi_status.strength
        self.label.set_label(f"{ssid} ({strength})% ")

    def on_network_speed(self, event):
        self.download_speed_label.set_label(event.download_speed)
        self.upload_speed_label.set_label(event.upload_speed)

    def on_network_list(self, event):
        self.popover.update(event.list)

    def on_click(self, _):
        self.popover.popup()


class NetworkPopover(Gtk.PopoverMenu):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.model = Gio.Menu.new()
        self.add_settings_row()
        self.add_ping_row()

        self.set_menu_model(self.model)
        self.set_has_arrow(False)

        action_group = Gio.SimpleActionGroup.new()

        action = Gio.SimpleAction.new("settings")
        action.connect("activate", self.on_settings_row_clicked)
        action_group.add_action(action)

        action = Gio.SimpleAction.new("ping")
        action.connect("activate", self.on_ping_row_clicked)
        action_group.add_action(action)

        action = Gio.SimpleAction.new("clicked", GLib.VariantType.new("s"))
        action.connect("activate", self.on_network_row_clicked)
        action_group.add_action(action)

        self.insert_action_group("network", action_group)

    def update(self, networks):
        self.model.remove_all()
        for network in networks:
            iface = network.iface
            address = network.address
            item = Gio.MenuItem.new(f"{iface}: {address}", None)
            item.set_action_and_target_value(
                "network.clicked", GLib.Variant("s", address)
            )
            self.model.append_item(item)
        self.add_settings_row()
        self.add_ping_row()

    def add_settings_row(self):
        item = Gio.MenuItem.new("Settings (nmtui)", "network.settings")
        self.model.append_item(item)

    def add_ping_row(self):
        item = Gio.MenuItem.new("Ping", "network.ping")
        self.model.append_item(item)

    def on_settings_row_clicked(self, action, parameter):
        Commands.spawn_network_editor()

    def on_ping_row_clicked(self, action, parameter):
        ctx.windows.ping.toggle()

    def on_network_row_clicked(self, action, parameter):
        ip = parameter.get_string()

        display = Gdk.Display.get_default()
        clipboard = display.get_clipboard()
        clipboard.set(ip)

        notification = Gio.Notification.new(f"Copied {ip}")
        ctx.app.send_notification(None, notification)
