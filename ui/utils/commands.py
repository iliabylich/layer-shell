from liblayer_shell_io import Commands as IoCommands
from utils.context import ctx


class CommandsMeta:
    def __getattr__(self, name):
        def _missing(*args, **kwargs):
            method = getattr(IoCommands, name)
            method(ctx.ui_ctx, *args, **kwargs)

        return _missing


Commands = CommandsMeta()
