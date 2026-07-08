use crate::utils::{ArrayWriter, StringRef, StringRefExt as _, getenv};
use alloc::vec::Vec;
use anyhow::{Context as _, Result, ensure};
use core::fmt::Write;
use toml_parser::{
    ErrorSink, ParseError, Raw, Source, Span, decoder::Encoding, parser::EventReceiver,
};

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) lock: StringRef,
    pub(crate) reboot: StringRef,
    pub(crate) shutdown: StringRef,
    pub(crate) logout: StringRef,
    pub(crate) edit_wifi: StringRef,
    pub(crate) edit_bluetooth: StringRef,
    pub(crate) open_system_monitor: StringRef,
    pub(crate) change_wallpaper: StringRef,

    pub(crate) ping: Vec<StringRef>,
    pub(crate) terminal: Terminal,
}
#[derive(Debug)]
pub(crate) struct Terminal {
    pub(crate) label: StringRef,
    pub(crate) command: Vec<StringRef>,
}

impl Config {
    pub(crate) fn read() -> Result<Self> {
        let mut buf = [0; 256];
        let mut writer = ArrayWriter::new(&mut buf);
        fmt_config_path(&mut writer)?;
        let path = writer.as_str()?;

        let fd = unsafe { libc::open(path.as_ptr().cast(), libc::O_RDONLY) };
        ensure!(fd != -1, "failed to open config at {path:?}");
        let fd = AutoCloseFd(fd);

        let mut buf = [0; 1_024];
        let res = unsafe { libc::read(fd.0, buf.as_mut_ptr().cast(), 1_024) };

        let len = usize::try_from(res).context("failed to read config")?;
        let buf = buf
            .get(..len)
            .context("reading config exceeded buffer size")?;

        let contents = core::str::from_utf8(buf).context("non-utf8 config")?;
        let config = Self::from_toml(contents)?;

        log::info!(target: "Config", "{config:#?}");

        Ok(config)
    }

    fn from_toml(contents: &str) -> Result<Self> {
        let source = Source::new(contents);
        let lexer = source.lex();
        let tokens = lexer.collect::<Vec<_>>();

        let mut errhandler = |error: ParseError| {
            log::error!("failed to parse TOML config: {error:?}");
            unsafe { libc::exit(1) };
        };

        let mut receiver = ConfigReceiver::new(contents);
        toml_parser::parser::parse_document(&tokens, &mut receiver, &mut errhandler);

        receiver.into_config()
    }
}

struct ConfigReceiver<'a> {
    source: Source<'a>,
    pending_key: Option<&'a str>,

    lock: Option<&'a str>,
    reboot: Option<&'a str>,
    shutdown: Option<&'a str>,
    logout: Option<&'a str>,
    edit_wifi: Option<&'a str>,
    edit_bluetooth: Option<&'a str>,
    open_system_monitor: Option<&'a str>,
    change_wallpaper: Option<&'a str>,
    ping: Option<&'a str>,
    terminal_label: Option<&'a str>,
    terminal_command: Option<&'a str>,
}

impl<'a> ConfigReceiver<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            source: Source::new(input),
            pending_key: None,
            lock: None,
            reboot: None,
            shutdown: None,
            logout: None,
            edit_wifi: None,
            edit_bluetooth: None,
            open_system_monitor: None,
            change_wallpaper: None,
            ping: None,
            terminal_label: None,
            terminal_command: None,
        }
    }

    fn into_config(self) -> Result<Config> {
        let lock = self.lock.context("no lock")?;
        let reboot = self.reboot.context("no reboot")?;
        let shutdown = self.shutdown.context("no shutdown")?;
        let logout = self.logout.context("no logout")?;
        let edit_wifi = self.edit_wifi.context("no edit_wifi")?;
        let edit_bluetooth = self.edit_bluetooth.context("no edit_bluetooth")?;
        let open_system_monitor = self.open_system_monitor.context("no open_system_monitor")?;
        let change_wallpaper = self.change_wallpaper.context("no change_wallpaper")?;
        let ping = self.ping.context("no ping")?;
        let terminal_label = self.terminal_label.context("no terminal_label")?;
        let terminal_command = self.terminal_command.context("no terminal_command")?;

        Ok(Config {
            lock: StringRef::new(lock),
            reboot: StringRef::new(reboot),
            shutdown: StringRef::new(shutdown),
            logout: StringRef::new(logout),
            edit_wifi: StringRef::new(edit_wifi),
            edit_bluetooth: StringRef::new(edit_bluetooth),
            open_system_monitor: StringRef::new(open_system_monitor),
            change_wallpaper: StringRef::new(change_wallpaper),
            ping: ping.split_whitespace().map(StringRef::new).collect(),
            terminal: Terminal {
                label: StringRef::new(terminal_label),
                command: terminal_command
                    .split_whitespace()
                    .map(StringRef::new)
                    .collect(),
            },
        })
    }
}

impl EventReceiver for ConfigReceiver<'_> {
    fn simple_key(&mut self, span: Span, _kind: Option<Encoding>, error: &mut dyn ErrorSink) {
        let Some(key) = self.source.get(span) else {
            error.report_error(ParseError::new("can't get source of span"));
            return;
        };
        if self.pending_key.is_some() {
            error.report_error(ParseError::new("unhandled key"));
            return;
        }
        self.pending_key = Some(key.as_str());
    }

    fn scalar(&mut self, span: Span, encoding: Option<Encoding>, error: &mut dyn ErrorSink) {
        let Some(key) = self.pending_key.take() else {
            error.report_error(ParseError::new("value without a key"));
            return;
        };
        let mut value = "";
        let Some(raw) = self
            .source
            .get(span)
            .map(|raw| Raw::new_unchecked(raw.as_str(), encoding, span))
        else {
            error.report_error(ParseError::new("failed to get source of span"));
            return;
        };
        let _ = raw.decode_scalar(&mut value, error);
        match key {
            "lock" => self.lock = Some(value),
            "reboot" => self.reboot = Some(value),
            "shutdown" => self.shutdown = Some(value),
            "logout" => self.logout = Some(value),
            "edit_wifi" => self.edit_wifi = Some(value),
            "edit_bluetooth" => self.edit_bluetooth = Some(value),
            "open_system_monitor" => self.open_system_monitor = Some(value),
            "change_wallpaper" => self.change_wallpaper = Some(value),
            "ping" => self.ping = Some(value),
            "terminal_label" => self.terminal_label = Some(value),
            "terminal_command" => self.terminal_command = Some(value),
            _ => {
                log::error!("unknown config key {key}");
                error.report_error(ParseError::new("unknown config key"));
            }
        }
    }

    fn key_val_sep(&mut self, _span: Span, _error: &mut dyn ErrorSink) {}
    fn whitespace(&mut self, _span: Span, _error: &mut dyn ErrorSink) {}
    fn comment(&mut self, _span: Span, _error: &mut dyn ErrorSink) {}
    fn newline(&mut self, _span: Span, _error: &mut dyn ErrorSink) {}

    fn error(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("got parse error"));
    }

    fn inline_table_open(&mut self, _span: Span, _error: &mut dyn ErrorSink) -> bool {
        true
    }
    fn array_open(&mut self, _span: Span, _error: &mut dyn ErrorSink) -> bool {
        true
    }

    fn std_table_open(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported std_table_open"));
    }
    fn std_table_close(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported std_table_close"));
    }
    fn array_table_open(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported array_table_open"));
    }
    fn array_table_close(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported array_table_close"));
    }
    fn inline_table_close(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported inline_table_close"));
    }
    fn array_close(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported array_close"));
    }
    fn key_sep(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported key_sep"));
    }
    fn value_sep(&mut self, _span: Span, error: &mut dyn ErrorSink) {
        error.report_error(ParseError::new("unsupported value_sep"));
    }
}

fn fmt_config_path(mut w: impl Write) -> Result<()> {
    let xdg_config_dir = getenv(c"XDG_CONFIG_DIR")
        .map(core::str::from_utf8)
        .transpose()
        .context("non-utf8 $XDG_CONFIG_DIR")?;
    let home = getenv(c"HOME")
        .map(core::str::from_utf8)
        .transpose()
        .context("non-utf8 $HOME")?
        .context("no $HOME")?;

    if let Some(xdg_config_dir) = xdg_config_dir {
        write!(&mut w, "{xdg_config_dir}/layer-shell/config.toml")?;
    } else {
        write!(&mut w, "{home}/.config/layer-shell/config.toml")?;
    }
    Ok(())
}

struct AutoCloseFd(i32);

impl Drop for AutoCloseFd {
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };
    }
}
