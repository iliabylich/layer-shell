use rusty_network_manager::NetworkManagerProxy;
use zbus::Connection;

use crate::utils::{singleton, Singleton};

pub(crate) struct NM {
    pub(crate) connection: Connection,
    pub(crate) nm: NetworkManagerProxy<'static>,
}
singleton!(NM);
impl NM {
    pub(crate) async fn start_if_not_started() {
        if NM::is_set() {
            return;
        }
        let connection = Connection::system()
            .await
            .expect("Could not get a connection.");

        let nm = NetworkManagerProxy::new(&connection)
            .await
            .expect("Could not get NetworkManager");

        Self::set(Self { connection, nm });
    }
}
