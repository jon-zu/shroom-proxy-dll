use hudhook::Hudhook;
use hudhook::hooks::{dx9::ImguiDx9Hooks, ImguiRenderLoop};
use imgui::{Condition, ImColor32, WindowHoveredFlags};
use windows::Win32::Foundation::HMODULE;

/*
pub fn init_module(hinst: HMODULE) {
    hudhook::lifecycle::global_state::set_module(HINSTANCE(hinst));
}*/

pub fn init_hooks(hmodule: HMODULE) -> anyhow::Result<()> {
    /*let hooks: Box<dyn hooks::Hooks> = { OverlayHook::new().into_hook::<ImguiDx9Hooks>() };
    unsafe { hooks.hook() };
    hudhook::lifecycle::global_state::set_hooks(hooks);*/
    Hudhook::builder()
        .with(OverlayHook::new().into_hook::<ImguiDx9Hooks>())
        .with_hmodule(hudhook::HINSTANCE(hmodule))
        .build()
        .apply()
        .map_err(|e| anyhow::anyhow!("Unable to apply hooks: {:?}", e))?;
    Ok(())
}

struct OverlayHook {
    clicks: usize,
    v: u32,
}

impl OverlayHook {
    fn new() -> Self {
        log::info!("Initializing dx9 overlay");
        OverlayHook { clicks: 0, v: 0 }
    }
}

impl ImguiRenderLoop for OverlayHook {
    fn initialize(&mut self, _ctx: &mut imgui::Context) {}
    fn render(&mut self, ui: &mut imgui::Ui) {
        let mouse_pos = ui.io().mouse_pos;
        ui.window("Overlay test")
            .size([450.0, 210.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Overlay");
                ui.slider("Slider", 0, 100, &mut self.v);
                ui.separator();
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1}) - clicks: {}",
                    mouse_pos[0], mouse_pos[1], self.clicks
                ));

                if ui.button("Click me") {
                    self.clicks += 1;
                }
            });
        // ToDo: find a way to (re)draw the og cursor
        if ui.is_window_hovered_with_flags(WindowHoveredFlags::ANY_WINDOW) {
            let draw_list = ui.get_foreground_draw_list();
            draw_list
                .add_circle(mouse_pos, 3., ImColor32::WHITE)
                .build();
        }
    }
}
