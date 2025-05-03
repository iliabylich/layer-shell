from gi.repository import Gtk


class Sound(Gtk.Box):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = app
        self.app.pub_sub.subscribe(self)

        self.set_orientation(Gtk.Orientation.HORIZONTAL)
        self.set_spacing(5)
        self.set_css_classes(["widget", "sound", "padded"])
        self.set_name("Sound")

        self.image = Gtk.Image.new_from_icon_name("dialog-question")
        self.append(self.image)

    def on_volume(self, event):
        volume = event.volume
        muted = event.muted
        icon_name = None
        if volume == 0 or muted:
            icon_name = "audio-volume-muted-symbolic"
        elif volume >= 1 and volume < 34:
            icon_name = "audio-volume-low-symbolic"
        elif volume >= 34 and volume < 67:
            icon_name = "audio-volume-medium-symbolic"
        elif volume >= 67 and volume < 95:
            icon_name = "audio-volume-high-symbolic"
        else:
            icon_name = "audio-volume-overamplified-symbolic"
        self.image.set_from_icon_name(icon_name)
