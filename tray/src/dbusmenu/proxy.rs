use zbus::proxy;

#[proxy(interface = "com.canonical.dbusmenu", assume_defaults = true)]
pub(crate) trait DBusMenu {
    fn about_to_show(&self, id: i32) -> zbus::Result<bool>;

    fn about_to_show_group(&self, ids: &[i32]) -> zbus::Result<(Vec<i32>, Vec<i32>)>;

    fn event(
        &self,
        id: i32,
        event_id: &str,
        data: &zbus::zvariant::Value<'_>,
        timestamp: u32,
    ) -> zbus::Result<()>;

    fn event_group(
        &self,
        events: &[&(i32, &str, &zbus::zvariant::Value<'_>, u32)],
    ) -> zbus::Result<Vec<i32>>;

    fn get_group_properties(
        &self,
        ids: &[i32],
        property_names: &[&str],
    ) -> zbus::Result<
        Vec<(
            i32,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        )>,
    >;

    fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: &[&str],
    ) -> zbus::Result<(
        u32,
        (
            i32,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
            Vec<zbus::zvariant::OwnedValue>,
        ),
    )>;

    fn get_property(&self, id: i32, name: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;

    #[zbus(signal)]
    fn item_activation_requested(&self, id: i32, timestamp: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn items_properties_updated(
        &self,
        updated_props: Vec<(
            i32,
            std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        )>,
        removed_props: Vec<(i32, Vec<&str>)>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    fn layout_updated(&self, revision: u32, parent: i32) -> zbus::Result<()>;

    #[zbus(property)]
    fn icon_theme_path(&self) -> zbus::Result<Vec<String>>;

    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn text_direction(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn version(&self) -> zbus::Result<u32>;
}
