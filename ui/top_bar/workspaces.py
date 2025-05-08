from gi.repository import Gtk
from utils.commands import Commands
from utils.context import ctx


class Workspaces(Gtk.Box):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        ctx.pub_sub.subscribe(self)
        self.workspaces_count = 10

        self.set_orientation(Gtk.Orientation.HORIZONTAL)
        self.set_spacing(0)
        self.set_css_classes(["widget", "workspaces"])
        self.set_name("Workspaces")

        self.buttons = []
        for idx in range(0, self.workspaces_count):
            button = Gtk.Button(label=f"{idx + 1}")
            self.append(button)
            self.buttons.append(button)
            button.connect("clicked", lambda _, idx=idx: self.on_click(idx))

    def on_workspaces(self, event):
        for idx in range(0, self.workspaces_count):
            button = self.buttons[idx]
            visible = (idx + 1) in event.ids
            button.set_visible(visible or idx < 5)
            if idx + 1 == event.active_id:
                button.set_css_classes(["active"])
            else:
                button.set_css_classes([])

    def on_click(self, idx):
        Commands.hyprland_go_to_workspace(idx)
