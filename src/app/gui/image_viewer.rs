use egui::Id;

#[derive(Default, Clone)]
pub struct ImageViewerState {
    pub fitted_to_view: bool,
    pub offset: egui::Vec2,
    pub zoom: f32,
}

pub fn fit_to_view(state: &mut ImageViewerState, rect: egui::Rect, texture: &egui::TextureHandle) {
    let image_size = texture.size_vec2();
    let view_size = rect.size();

    let scale_x = view_size.x / image_size.x;
    let scale_y = view_size.y / image_size.y;

    state.zoom = scale_x.min(scale_y);
    state.zoom = state.zoom.clamp(0.01, 100.0);

    state.offset = egui::Vec2::ZERO;
}

pub fn image_viewer(id: Id, ui: &mut egui::Ui, texture: &egui::TextureHandle) {
    let mut state = ui
        .ctx()
        .data_mut(|data| data.get_persisted::<ImageViewerState>(id))
        .unwrap_or_default();

    let available = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available, egui::Sense::drag());

    if !state.fitted_to_view {
        fit_to_view(&mut state, rect, texture);
        state.fitted_to_view = true;
    }

    if response.dragged() {
        state.offset += response.drag_delta();
    }

    ui.ctx().input(|input| {
        let zoom_delta = input.zoom_delta();
        state.zoom *= zoom_delta;
        state.zoom = state.zoom.clamp(0.1, 10.0);
    });

    let image_size = texture.size_vec2() * state.zoom;
    let image_rect = egui::Rect::from_center_size(rect.center() + state.offset, image_size);

    egui::Image::new(texture).paint_at(ui, image_rect);

    ui.ctx().data_mut(|data| {
        data.insert_persisted(id, state);
    });
}
