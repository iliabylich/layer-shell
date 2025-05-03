from math import floor

from gi.repository import Gtk


class CPU(Gtk.Box):
    def __init__(self, app, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.cpu_count = 12
        self.app = app
        self.app.pub_sub.subscribe(self)

        self.set_orientation(Gtk.Orientation.HORIZONTAL)
        self.set_spacing(3)
        self.set_css_classes(["widget", "cpu", "padded"])
        self.set_name("CPU")

        self.labels = []
        for _ in range(0, self.cpu_count):
            label = CpuLabel()
            self.labels.append(label)
            self.append(label)

    def on_cpu_usage(self, event):
        for i in range(0, self.cpu_count):
            label = self.labels[i]
            load = event.usage_per_core[i]
            label.set_load(load)


INDICATORS = [
    "<span color='#FFFFFF'>▁</span>",
    "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>",
    "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>",
    "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>",
    "<span color='#E60000'>█</span>",
]


class CpuLabel(Gtk.Label):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.set_use_markup(True)
        self.set_load(0)

    def set_load(self, load):
        indicator_idx = floor(load / 100.0 * len(INDICATORS))

        if indicator_idx == len(INDICATORS):
            indicator_idx -= 1

        markup = INDICATORS[indicator_idx]
        self.set_label(markup)
