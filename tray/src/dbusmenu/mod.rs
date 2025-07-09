mod items_properties_updated;
mod layout;
mod layout_updated;
mod proxy;

pub(crate) use items_properties_updated::ItemsPropertiesUpdated;
pub(crate) use layout::Layout;
pub(crate) use layout_updated::LayoutUpdated;

pub(crate) async fn trigger_tray_item(
    conn: zbus::Connection,
    service: String,
    menu: String,
    id: i32,
) -> anyhow::Result<()> {
    use proxy::DBusMenuProxy;

    let proxy = DBusMenuProxy::builder(&conn)
        .destination(service.to_string())?
        .path(menu)?
        .build()
        .await?;

    let data = zbus::zvariant::Value::I32(0);
    let timestamp = match u32::try_from(chrono::Utc::now().timestamp()) {
        Ok(ts) => ts,
        Err(err) => {
            log::error!(target: "Tray", "can't construct u32 from chrono timestamp: {err:?}");
            1750950284
        }
    };

    proxy.event(id, "clicked", &data, timestamp).await?;

    Ok(())
}
