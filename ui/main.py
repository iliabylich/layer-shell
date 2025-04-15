#!/usr/bin/env python3

from setproctitle import setproctitle
import setup_libs
from app import App

setproctitle("layer-shell")

app = App(application_id="org.me.LayerShell")
app.run(None)
