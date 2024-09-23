use gtk4::{
    glib::Object,
    prelude::{IsA, WidgetExt},
};

use crate::globals::globalize_widget;

pub(crate) trait TypedChildren {
    fn children_as<const N: usize, C>(&self) -> [&'static C; N]
    where
        C: IsA<Object>;

    fn first_child_as<C>(&self) -> &'static C
    where
        C: IsA<Object>;

    fn last_child_as<C>(&self) -> &'static C
    where
        C: IsA<Object>;
}

impl<T> TypedChildren for T
where
    T: WidgetExt,
{
    fn children_as<const N: usize, C>(&self) -> [&'static C; N]
    where
        C: IsA<Object>,
    {
        let mut out = vec![];
        let mut child = self.first_child();
        while let Some(widget) = child {
            out.push(globalize_widget(&widget));
            child = widget.next_sibling();
        }
        out.try_into().unwrap()
    }

    fn first_child_as<C>(&self) -> &'static C
    where
        C: IsA<Object>,
    {
        globalize_widget(&self.first_child().unwrap())
    }

    fn last_child_as<C>(&self) -> &'static C
    where
        C: IsA<Object>,
    {
        globalize_widget(&self.last_child().unwrap())
    }
}
