class Context:
    def __init__(self):
        self.app = None

        self.io_ctx = None
        self.ui_ctx = None

        self.icons = None

        self.windows = Windows()
        self.pub_sub = None


class Windows:
    def __init__(self):
        self.htop = None
        self.launcher = None
        self.ping = None
        self.top_bar = None
        self.weather = None
        self.session = None


ctx = Context()
