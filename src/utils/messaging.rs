use zbus::{interface, proxy};

use crate::globals::toggle_window;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum DBusMessage {
    Stop,
    ToggleLauncher,
    ToggleLogoutScreen,
}

impl DBusMessage {
    fn execute(self) {
        match self {
            DBusMessage::Stop => std::process::exit(0),
            DBusMessage::ToggleLauncher => toggle_window("Launcher"),
            DBusMessage::ToggleLogoutScreen => toggle_window("LogoutScreen"),
        }
    }
}

struct Listener;

#[interface(name = "com.me.LayerShellBus")]
impl Listener {
    // Can be `async` as well.
    fn dispatch(&mut self, message: &str) {
        let message: DBusMessage = serde_json::from_str(message).unwrap();
        message.execute();
    }
}

#[proxy(
    default_service = "com.me.LayerShellBus",
    default_path = "/com/me/LayerShellBus",
    interface = "com.me.LayerShellBus"
)]
trait Publisher {
    fn dispatch(&self, message: &str) -> zbus::Result<()>;
}

pub(crate) struct DBus {}

impl DBus {
    pub(crate) fn subscribe() {
        gtk4::glib::spawn_future_local(async move {
            let listener = Listener;
            let _conn = zbus::connection::Builder::session()
                .unwrap()
                .name("com.me.LayerShellBus")
                .unwrap()
                .serve_at("/com/me/LayerShellBus", listener)
                .unwrap()
                .build()
                .await
                .unwrap();

            // keep spinning forever
            std::future::pending::<()>().await;
        });
    }

    pub(crate) fn send(message: DBusMessage) {
        let message = serde_json::to_string(&message).unwrap();
        let connection = zbus::blocking::Connection::session().unwrap();
        let proxy = PublisherProxyBlocking::new(&connection).unwrap();
        let _ = proxy.dispatch(&message);
    }
}
