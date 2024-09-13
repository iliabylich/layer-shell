use gtk4::Window;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub(crate) struct LayerOptions<'a> {
    layer: Layer,
    anchors: &'a [Edge],
    margins: &'a [(Edge, i32)],
    namespace: Option<&'a str>,
    keyboard_mode: Option<KeyboardMode>,
}

impl<'a> LayerOptions<'a> {
    pub(crate) fn builder() -> LayerOptionsBuilder<'a> {
        LayerOptionsBuilder::default()
    }
}

#[derive(Default)]
pub(crate) struct LayerOptionsBuilder<'a> {
    layer: Option<Layer>,
    anchors: Option<&'a [Edge]>,
    margins: Option<&'a [(Edge, i32)]>,
    namespace: Option<&'a str>,
    keyboard_mode: Option<KeyboardMode>,
}

impl<'a> LayerOptionsBuilder<'a> {
    pub(crate) fn with_layer(mut self, value: Layer) -> Self {
        self.layer = Some(value);
        self
    }
    pub(crate) fn with_anchors(mut self, anchors: &'a [Edge]) -> Self {
        self.anchors = Some(anchors);
        self
    }
    pub(crate) fn with_margins(mut self, value: &'a [(Edge, i32)]) -> Self {
        self.margins = Some(value);
        self
    }
    pub(crate) fn with_namespace(mut self, value: &'a str) -> Self {
        self.namespace = Some(value);
        self
    }
    pub(crate) fn with_keyboard_mode(mut self, value: KeyboardMode) -> Self {
        self.keyboard_mode = Some(value);
        self
    }

    pub(crate) fn build(self) -> LayerOptions<'a> {
        LayerOptions {
            layer: self.layer.unwrap(),
            anchors: self.anchors.unwrap_or_default(),
            margins: self.margins.unwrap_or_default(),
            namespace: self.namespace,
            keyboard_mode: self.keyboard_mode,
        }
    }
}

pub(crate) fn layer_window(window: &Window, options: LayerOptions) {
    LayerShell::init_layer_shell(window);
    LayerShell::set_layer(window, options.layer);
    for edge in options.anchors {
        LayerShell::set_anchor(window, *edge, true);
    }
    for (edge, margin) in options.margins {
        LayerShell::set_margin(window, *edge, *margin);
    }
    if let Some(namespace) = options.namespace {
        LayerShell::set_namespace(window, namespace);
    }
    if let Some(keyboard_mode) = options.keyboard_mode {
        LayerShell::set_keyboard_mode(window, keyboard_mode)
    }
}
